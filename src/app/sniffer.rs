// https://www.rfc-editor.org/rfc/rfc4346#page-17

// struct {
//     uint8 major, minor;
// } ProtocolVersion;

// enum {
//     change_cipher_spec(20), alert(21), handshake(22),
//     application_data(23), (255)
// } ContentType;

// struct {
//     ContentType type;
//     ProtocolVersion version;
//     uint16 length;
//     opaque fragment[TLSPlaintext.length];
// } TLSPlaintext;

// ClientHello 由 record + Handshake Client Hello Request 共同组成

use std::{
    cmp::min,
    io,
    ops::Range,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
    u8,
};

use byteorder::{BigEndian, ByteOrder};
use libc::truncate;
use log::debug;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf},
    net::TcpStream,
    time::timeout,
};

use crate::proxy::StreamWrapperTrait;

// --------------------------------------------------------------|
// | 0x00, 0x03 | 0x00, 0x00, 0x00| 0x01, 0x01, 0x01, 0x01, 0x01 |
// |<--2 bytes->|<----3 bytes---->|<-------remaining data------->|
//  range
// 保留 3 bytes
fn slice_at_range(data: &[u8], range: Range<usize>) -> Result<&[u8], String> {
    let len_in_bits = data
        .get(range.clone())
        .ok_or("failed to get data length field from range")?;

    let mut len_behind_length_indicator = 0usize;
    for bit in len_in_bits {
        len_behind_length_indicator = len_behind_length_indicator << 8 | (*bit as usize);
    }
    return data
        .get(range.end..range.end + len_behind_length_indicator)
        .ok_or("failed to slice data".to_string());
}

// --------------------------------------------------------------|
// | 0x00, 0x03 | 0x00, 0x00, 0x00| 0x01, 0x01, 0x01, 0x01, 0x01 |
// |<--2 bytes->|<----3 bytes---->|<-------remaining data------->|
//  range
// 保留 remaining data
fn truncate_before(data: &[u8], range: Range<usize>) -> Result<&[u8], String> {
    let len = slice_at_range(data, range.clone())?.len();
    Ok(&data[range.end + len..])
}

macro_rules! truncate {
    ($x: expr) => {
        match $x {
            Ok(x) => x,
            Err(s) => {
                debug!("bad tls record {}", s);
                return Err(io::Error::new(io::ErrorKind::Other, s));
            }
        }
    };
}

pub struct Sniffer<T> {
    stream: T,
    buf: Vec<u8>,
}

impl<T> Sniffer<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn sniff(&mut self) -> io::Result<Option<String>> {
        // let buf0: Vec<u8> = Vec::with_capacity(2048);
        // assert!(buf0.len() == 0);
        // assert!((&*buf0).len() == 0);

        // with_capacity len 是 0，所以 deref 时 [u8] len 也是 0
        // stream.read 接受 ReadBuf，ReadBuf 会从 [u8] 初始化，导致 [u8] len 也是 0
        // 最终ReadBuf#remaining() 始终为 0，ReadBuf#put_slice 写不进数据 
        
        // 而 vec! 会将 len 设置为 2048
        // let buf1 = vec![0u8; 2048];
        // assert!(buf1.len() != 0);
        // assert!((&*buf1).len() != 0);

        let mut buf = vec![0u8; 2048];
        let wait = Duration::from_millis(500);
        for i in 1..3 {
            match timeout(wait, self.stream.read(&mut buf)).await? {
                // https://www.rfc-editor.org/rfc/rfc4346#page-17
                Ok(n) => {
                    // 需要存储全部TLS record数据
                    // 当连接server时发过去
                    self.buf.extend_from_slice(&*buf);
                    let curr = &self.buf[..];

                    if curr.len() < 5 {
                        debug!("read len {}, continue", curr.len());
                        continue;
                    }
                    if curr[0] != 0x16 {
                        // content type
                        debug!("not handshake record {}", curr[0]);
                        return Ok(None);
                    }
                    if curr[1] != 0x03 {
                        // major version
                        debug!("major version should be 3");
                        return Ok(None);
                    }
                    let client_hello_length = BigEndian::read_u16(&curr[3..5]) as usize;
                    if curr.len() < 5 + client_hello_length {
                        // client hello 不完整，需要继续读
                        continue;
                    }
                    // Handshake Protocol Client Hello
                    let curr = &curr[5..];
                    if curr[0] != 0x01 {
                        debug!("not client hello! {}", curr[0]);
                        return Ok(None);
                    }
                    // session id length
                    let curr = truncate!(truncate_before(&curr, 38..39));
                    // cipher suites length
                    let curr = truncate!(truncate_before(&curr, 0..2));
                    // compression methods length
                    let curr = truncate!(truncate_before(&curr, 0..1));
                    // extensions
                    let curr = truncate!(slice_at_range(&curr, 0..2));
                    let mut extensions = curr;
                    // type(2 bytes) + length(2 bytes) == 4 bytes
                    while extensions.len() > 4 {
                        let ext_type = BigEndian::read_u16(&extensions[0..2]);
                        let extension = truncate!(slice_at_range(&extensions, 2..4));
                        if ext_type == 0 {
                            let server_name_bytes = truncate!(slice_at_range(&extension, 3..5));
                            let server_name = String::from_utf8_lossy(&server_name_bytes).into();
                            debug!("tls record sni {}", server_name);
                            return Ok(Some(server_name));
                        } else {
                            extensions = truncate!(truncate_before(&extensions, 2..4));
                        }
                    }
                }
                Err(err) => {
                    debug!("{}", err);
                    return Err(err);
                }
            }
        }
        Ok(None)
    }
    pub fn new(stream: T) -> Sniffer<T> {
        Sniffer {
            stream,
            buf: Vec::with_capacity(2048),
        }
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Sniffer<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if !self.buf.is_empty() {
            // 将 client hello 写到 server
            let accepted_len = min(self.buf.len(), buf.remaining());
            buf.put_slice(&self.buf[..accepted_len]);
            self.buf.drain(..accepted_len);
            Poll::Ready(Ok(()))
        } else {
            AsyncRead::poll_read(self, cx, buf)
        }
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Sniffer<T> {
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.stream), cx)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.stream), cx)
    }
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.stream), cx, buf)
    }
}

#[tokio::test]
async fn test_server_name() {
    let str = [
        0x16, 0x03, 0x01, 0x02, 0x00, 0x01, 0x00, 0x01, 0xfc, 0x03, 0x03, 0x23, 0x9d, 0x18, 0x59,
        0x78, 0x08, 0x33, 0xd0, 0xd2, 0x12, 0xdc, 0x2e, 0x33, 0xaf, 0xa0, 0x48, 0xdb, 0x76, 0x1a,
        0x11, 0x4b, 0x68, 0xcb, 0x91, 0xbd, 0x7c, 0xf9, 0xc4, 0x6d, 0xdb, 0xce, 0xc0, 0x20, 0x76,
        0x2c, 0x00, 0x00, 0x8f, 0x77, 0x33, 0x8f, 0x74, 0xcc, 0x85, 0xfc, 0xe6, 0x57, 0x5c, 0x7b,
        0xff, 0x67, 0xfc, 0xa9, 0xd2, 0x90, 0x84, 0x50, 0x35, 0x96, 0x84, 0x60, 0x55, 0x5a, 0x48,
        0xa5, 0x00, 0x20, 0xca, 0xca, 0x13, 0x01, 0x13, 0x02, 0x13, 0x03, 0xc0, 0x2b, 0xc0, 0x2f,
        0xc0, 0x2c, 0xc0, 0x30, 0xcc, 0xa9, 0xcc, 0xa8, 0xc0, 0x13, 0xc0, 0x14, 0x00, 0x9c, 0x00,
        0x9d, 0x00, 0x2f, 0x00, 0x35, 0x01, 0x00, 0x01, 0x93, 0x3a, 0x3a, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0d, 0x00, 0x0b, 0x00, 0x00, 0x08, 0x63, 0x2e, 0x6d, 0x73, 0x6e, 0x2e, 0x63, 0x6e,
        0x00, 0x17, 0x00, 0x00, 0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x0a, 0x00, 0x08,
        0xfa, 0xfa, 0x00, 0x1d, 0x00, 0x17, 0x00, 0x18, 0x00, 0x0b, 0x00, 0x02, 0x01, 0x00, 0x00,
        0x23, 0x00, 0x00, 0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, 0x68, 0x32, 0x08, 0x68, 0x74,
        0x74, 0x70, 0x2f, 0x31, 0x2e, 0x31, 0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0d, 0x00, 0x12, 0x00, 0x10, 0x04, 0x03, 0x08, 0x04, 0x04, 0x01, 0x05, 0x03, 0x08,
        0x05, 0x05, 0x01, 0x08, 0x06, 0x06, 0x01, 0x00, 0x12, 0x00, 0x00, 0x00, 0x33, 0x00, 0x2b,
        0x00, 0x29, 0xfa, 0xfa, 0x00, 0x01, 0x00, 0x00, 0x1d, 0x00, 0x20, 0x32, 0xd2, 0x4c, 0x0a,
        0xf6, 0x24, 0x83, 0x88, 0x2d, 0x3c, 0xbb, 0x0c, 0xec, 0x17, 0x3a, 0x24, 0xd1, 0xad, 0x2a,
        0xd3, 0xa7, 0x67, 0x56, 0x19, 0x02, 0x25, 0x6b, 0xf2, 0x9c, 0x54, 0x25, 0x2e, 0x00, 0x2d,
        0x00, 0x02, 0x01, 0x01, 0x00, 0x2b, 0x00, 0x07, 0x06, 0xba, 0xba, 0x03, 0x04, 0x03, 0x03,
        0x00, 0x1b, 0x00, 0x03, 0x02, 0x00, 0x02, 0x44, 0x69, 0x00, 0x05, 0x00, 0x03, 0x02, 0x68,
        0x32, 0x2a, 0x2a, 0x00, 0x01, 0x00, 0x00, 0x15, 0x00, 0xcf, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    struct FakeStream<'a> {
        value: &'a [u8]
    }
    impl AsyncRead for FakeStream<'_> {
        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
            let remaining = min(buf.remaining(), self.value.len());
            buf.put_slice(&self.value[..remaining]);
            self.value = &self.value[remaining..];
            Poll::Ready(Ok(()))
        }
    }
    impl AsyncWrite for FakeStream<'_> {
        fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
            unimplemented!()
        }
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
            unimplemented!()
        }
        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
            unimplemented!()
        }
    }
    let s = FakeStream{
        value: &str
    };
    let mut sniffer = Sniffer::new(s);
    let res = match sniffer.sniff().await {
        Ok(res) => res.unwrap(),
        Err(err) => {
            panic!(err);
        }
    };
    assert!("c.msn.cn" == res.as_str());
}
