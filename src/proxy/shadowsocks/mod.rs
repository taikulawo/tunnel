use bytes::BytesMut;
use core::fmt;
use futures::ready;
use lazy_static::lazy_static;
use md5::{Digest, Md5};
use std::{
    collections::HashMap,
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf, AsyncWrite};

use self::cipher::{Method, INFOS};

mod cipher;


enum ReadState {
    // 开始阶段，等待协议开头的salt
    WaitingSalt,
    WaitingChunk
}

enum WriteState {
    WaitingSalt,
    WaitingChunk
}
// shadowsocks 协议分析
// https://chaochaogege.com/2022/05/24/58/
struct ShadowsocksStream<T> {
    stream: T,
    read_buf: BytesMut,
    write_buf: BytesMut,
    read_state: ReadState,
    write_state: WriteState,
}

// https://github.com/v2fly/v2ray-core/blob/ca5695244c383870aed1976a59ae6e5eda94f999/proxy/shadowsocks/config.go#L228

impl<T> ShadowsocksStream<T> {
    pub fn new(stream: T, method: Method, password: String) -> io::Result<Self> {
        Ok(Self {
            stream,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            read_state: ReadState::WaitingSalt,
            write_state: WriteState::WaitingSalt
        })
    }
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
        match self.read_state {
            ReadState::WaitingSalt => {

            }
            ReadState::WaitingChunk => {
                
            }
        }
        // ready!(Pin::new(&mut self.stream));
        todo!()
    }
}

impl<T> AsyncWrite for ShadowsocksStream<T>
where
    T: Unpin + AsyncWrite
{
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        match self.write_state {
            WriteState::WaitingSalt => {

            }
            WriteState::WaitingChunk => {

            }
        }
        todo!()
    }
}

#[test]
fn stream_test() {
    let x = Method::AES_192_GCM;
    println!("{}", x.to_string());
}
