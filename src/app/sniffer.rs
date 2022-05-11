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

use std::{io, ops::Range};

use tokio::io::{AsyncWrite, AsyncRead};

pub struct Sniffer<T> {
    stream: T
}

pub fn truncate(data: &[u8], range: Range<usize>) -> Result<&[u8], &'static str> {
    let rangeData = data.get(range).ok_or("failed to decode")?;
    // rangeD
    todo!()
}

impl<T> Sniffer<T> 
where T: AsyncRead + AsyncWrite + Unpin
{
    pub async fn sniff(&mut self) -> io::Result<String> {
        todo!()
    }
}