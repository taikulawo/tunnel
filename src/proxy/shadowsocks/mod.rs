use bytes::{Buf, BufMut, BytesMut};
use futures::ready;
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use self::cipher::{
    password_to_cipher_key, AEADCipher, AeadDecryptor, AeadEncryptor, Method, INFOS,
};

mod cipher;

const MAX_PAYLOAD_LEN: usize = 0x3fff;

enum ReadState {
    // 开始阶段，等待协议开头的salt
    WaitingSalt,
    WaitingLength,
    WaitingPayload(usize),
}

enum WriteState {
    WaitingSalt,
    WaitingChunk,
    WritingChunk(usize),
}
// shadowsocks 协议分析
// https://chaochaogege.com/2022/05/24/58/
// 针对 0x3fff 的处理
// poll_write 要求返回 how many bytes written，所以caller poll_write之后需将buf切片，未写完的数据继续写
// ShadowsocksStream 只需要一次处理少于 0x3fff 的数据即可
//
// https://github.com/iamwwc/shadowsocks-rust/blob/218c6ec0e302977212ed4f8ec4816337780789aa/crates/shadowsocks/src/relay/tcprelay/proxy_stream/client.rs#L210
// shadowsocks-rust encrypted poll_write 多次循环全部将数据写完，而不是返回一次最多写入的bytes
struct ShadowsocksStream<T> {
    stream: T,
    read_buf: BytesMut,
    read_state: ReadState,

    write_buf: BytesMut,
    write_state: WriteState,

    psk: Vec<u8>,
    // WaitingSalt 阶段才能初始化
    cipher: AEADCipher,
    encryptor: Option<AeadEncryptor>,
    decryptor: Option<AeadDecryptor>,
}

// https://github.com/v2fly/v2ray-core/blob/ca5695244c383870aed1976a59ae6e5eda94f999/proxy/shadowsocks/config.go#L228

impl<T> ShadowsocksStream<T> {
    /// method:
    /// 1. aes-128-gcm
    /// 2. aes-256-gcm
    pub fn new(stream: T, method: &str, password: String) -> io::Result<Self> {
        let m = INFOS.get(&method).unwrap();
        let strong_password = password_to_cipher_key(&*password, m.key_len)?;
        let cipher = AEADCipher::new(m.algorithm);
        Ok(Self {
            stream,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            read_state: ReadState::WaitingSalt,
            write_state: WriteState::WaitingSalt,
            cipher,
            encryptor: None,
            decryptor: None,
            psk: strong_password,
        })
    }
}

impl<T> ShadowsocksStream<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read_exact(&mut self, cx: &mut Context<'_>, size: usize) -> Poll<io::Result<usize>> {
        while self.read_buf.len() < size {
            let len = self.read_buf.len();
            let additional = size - len;
            self.read_buf.reserve(additional);
            let dst_buf = unsafe {
                // [u8] => *[u8] => *[MaybeUninit<u8>] => [MaybeUninit<u8>] => &mut [MaybeUninit<u8>]
                &mut *(&mut self.read_buf[len..len + additional] as *mut _ as *mut _)
            };
            let mut read_buf = ReadBuf::uninit(dst_buf);
            ready!(Pin::new(&mut self.stream).poll_read(cx, &mut read_buf))?;
            let n = read_buf.filled().len();
            if n == 0 {
                //  If the difference is 0, EOF has been reached.
                if self.read_buf.is_empty() {
                    // ok
                    return Ok(0).into();
                } else {
                    // read_buf还有数据，但 read 却返回0，说明 EOF
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF!")).into();
                }
            }
            self.read_buf.advance(n);
        }
        Ok(size).into()
    }
}

fn map_crypto_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "crypto error")
}

impl<T> AsyncRead for ShadowsocksStream<T>
where
    T: Unpin + AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            match self.read_state {
                ReadState::WaitingSalt => {
                    let salt_len = self.cipher.key_len();
                    ready!(self.poll_read_exact(cx, salt_len))?;
                    let decryptor = self
                        .cipher
                        .decryptor(&self.psk, &self.read_buf[..salt_len])
                        .map_err(|x| map_crypto_error())?;
                    self.decryptor.replace(decryptor);
                    self.read_buf.clear();
                    self.read_state = ReadState::WaitingLength;
                }
                ReadState::WaitingLength => {
                    let tag_len = self.cipher.tag_len();
                    let encrypted_length_field_len = 2 + tag_len;
                    ready!(self.poll_read_exact(cx, encrypted_length_field_len))?;
                    // decryptor should always be Some
                    let me = &mut *self;
                    let dec = me.decryptor.as_mut().unwrap();
                    dec.decrypt(&mut me.read_buf)
                        .map_err(|x| map_crypto_error())?;
                    let buf = &self.read_buf;
                    let n = u16::from_be_bytes([buf[0], buf[1]]);
                    self.read_state = ReadState::WaitingPayload(n as usize);
                    self.read_buf.clear();
                }
                ReadState::WaitingPayload(n) => {
                    let encrypted_payload_field_len = self.cipher.tag_len() + n;
                    ready!(self.poll_read_exact(cx, encrypted_payload_field_len))?;
                    let me = &mut *self;
                    let dec = me.decryptor.as_mut().unwrap();
                    dec.decrypt(&mut me.read_buf)
                        .map_err(|x| map_crypto_error())?;
                    assert!(n == self.read_buf.len());
                    let remaining = usize::min(buf.remaining(), self.read_buf.len());
                    buf.put_slice(&self.read_buf[..remaining]);
                    if remaining < n {
                        self.read_state = ReadState::WaitingPayload(n - remaining);
                    } else {
                        self.read_buf.clear();
                        self.read_state = ReadState::WaitingLength;
                    }
                    return Ok(()).into();
                }
            }
        }
    }
}

impl<T> AsyncWrite for ShadowsocksStream<T>
where
    T: Unpin + AsyncWrite,
{
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let me = &mut *self;
        buf = &buf[..MAX_PAYLOAD_LEN];
        loop {
            match me.write_state {
                WriteState::WaitingSalt => {
                    let salt_len = me.cipher.key_len();
                    // https://github.com/v2fly/v2ray-core/blob/0746740b1072185634ef0873f1607f922a28efea/proxy/shadowsocks/protocol.go#L104
                    // secure random number
                    let mut srn = rand::rngs::StdRng::from_entropy();
                    for i in 0..salt_len {
                        me.write_buf[i] = srn.gen();
                    }
                    let encryptor = me
                        .cipher
                        .encryptor(&me.psk, &me.write_buf[..salt_len])
                        .map_err(|x| map_crypto_error())?;
                    // salt 写走
                    ready!(Pin::new(&mut me.stream).poll_write(cx, &me.write_buf[..salt_len]))?;
                    me.write_buf.clear();
                    me.encryptor.replace(encryptor);
                    me.write_state = WriteState::WaitingChunk;
                }
                WriteState::WaitingChunk => {
                    // length(2) tag(x) + payload(length) tag(x)
                    let encrypted_payload_length = 2 + me.cipher.tag_len() * 2 + buf.len();
                    let real_payload_len = buf.len() as u16;

                    me.write_buf.reserve(encrypted_payload_length);
                    me.write_buf.put_slice(&u16::to_be_bytes(real_payload_len));
                    let enc = me.encryptor.as_mut().unwrap();
                    enc.encrypt(&mut me.write_buf)
                        .map_err(|x| map_crypto_error())?;

                    unsafe {
                        me.write_buf.set_len(2 + me.cipher.tag_len());
                    }
                    me.write_buf.put_slice(&buf);
                    enc.encrypt(&mut me.write_buf)
                        .map_err(|x| map_crypto_error())?;
                    unsafe { me.write_buf.set_len(encrypted_payload_length) };
                    me.write_state = WriteState::WritingChunk(0);
                }
                WriteState::WritingChunk(ref mut total_len) => {
                    // write all
                    // so always return Ok(buf.len())
                    while (*total_len < me.write_buf.len()) {
                        let n =
                            ready!(Pin::new(&mut me.stream)
                                .poll_write(cx, &me.write_buf[*total_len..]))?;
                        *total_len += n;
                    }
                    me.write_buf.clear();
                    me.write_state = WriteState::WaitingChunk;
                    return Ok(buf.len()).into();
                }
            }
        }
    }
}

pub struct ShadowsocksDatagram {
    psk: Vec<u8>,
    cipher: AEADCipher,
}

// https://github.com/v2fly/v2ray-core/blob/0746740b1072185634ef0873f1607f922a28efea/proxy/shadowsocks/protocol.go#L185
// original [salt][target_address][forwarded data]
// encrypted [salt][encrypted_length][tag][    encrypted_payload            ][tag]
//                                        <-target_address || forwarded data->
// nonce 都从1开始，但由于UDP只使用一次，所以nonce一直都是 1 ？
// https://github.com/v2fly/v2ray-core/blob/3ef7feaeaf737d05c5a624c580633b7ce0f0f1be/common/crypto/auth.go#L73

// Since shadowsocks.org offline, I can't found original UDP spec for shadowsocks ever.
// following spec copy from https://github-wiki-see.page/m/shadowsocks/shadowsocks-org/wiki/AEAD-Ciphers
// An AEAD encrypted UDP packet has the following structure

// [salt][encrypted payload][tag]
// The salt is used to derive the per-session subkey and must be generated randomly to ensure uniqueness. Each UDP packet is encrypted/decrypted independently, using the derived subkey and a nonce with all zero byte
impl ShadowsocksDatagram {
    /// method:
    /// 1. aes-128-gcm
    /// 2. aes-256-gcm
    pub fn new(method: & str, password: &str) -> io::Result<Self> {
        let m = INFOS.get(&method).unwrap();
        let strong_password = password_to_cipher_key(password, m.key_len)?;
        let cipher = AEADCipher::new(m.algorithm);
        Ok(Self {
            cipher,
            psk: strong_password,
        })
    }
    pub fn encrypt(&self, mut buf: BytesMut) -> io::Result<Vec<u8>> {
        // generate salt
        let salt_len = self.cipher.key_len();
        let mut encrypted_buf = Vec::new();
        let mut rng = StdRng::from_entropy();
        for i in 0..salt_len {
            encrypted_buf[i] = rng.gen();
        }
        let mut encryptor = self
            .cipher
            .encryptor(&self.psk, &encrypted_buf[..salt_len])
            .map_err(|x| map_crypto_error())?;
        encryptor
            .encrypt(&mut buf)
            .map_err(|x| map_crypto_error())?;
        encrypted_buf.extend_from_slice(&buf);
        Ok(encrypted_buf)
    }

    pub fn decrypt(&self, mut buf: BytesMut) -> io::Result<Vec<u8>> {
        let salt_len = self.cipher.key_len();
        let salt = &buf[..salt_len];
        let mut decryptor = self
            .cipher
            .decryptor(&self.psk, salt)
            .map_err(|x| map_crypto_error())?;
        let decrypted_data = &mut buf.split_off(salt_len);
        let before_decrpyt_len = decrypted_data.len();
        decryptor
            .decrypt(decrypted_data)
            .map_err(|x| map_crypto_error())?;
        let valid_data_len = before_decrpyt_len - self.cipher.tag_len();
        let mut decrypted_buf = Vec::with_capacity(valid_data_len);
        decrypted_buf.extend_from_slice(&decrypted_data[..valid_data_len]);
        Ok(decrypted_buf)
    }
}

#[test]
fn stream_test() {
    let x = Method::AES_192_GCM;
    println!("{}", x.to_string());
}
