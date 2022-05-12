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

use std::{io, ops::Range, time::Duration, u8};

use byteorder::{BigEndian, ByteOrder};
use libc::truncate;
use log::debug;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    net::TcpStream,
    time::timeout,
};

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
pub fn truncate_before(data: &[u8], range: Range<usize>) -> Result<&[u8], String> {
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
        let mut buf = Vec::with_capacity(2048);
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
                            return Ok(Some(String::from_utf8_lossy(&server_name_bytes).into()));
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
