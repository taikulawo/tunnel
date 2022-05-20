use pin_project::pin_project;

use std::pin::Pin;
use std::task::Poll;
use std::{io, net::SocketAddr, task::Context};
use tokio::io::{AsyncWrite, ReadBuf};
use tokio::{
    io::AsyncRead,
    net::{TcpStream},
};


use crate::proxy::{create_bounded_tcp_socket};


#[pin_project]
pub struct ProxyStream {
    #[pin]
    inner: TcpStream,
}

impl ProxyStream {
    // connect will bypass tun routes, always directly connect
    pub async fn connect(addr: SocketAddr) -> io::Result<ProxyStream> {
        let socket = create_bounded_tcp_socket(addr.clone())?;
        let stream = socket.connect(addr).await?;
        Ok(ProxyStream { inner: stream })
    }
}

impl AsyncRead for ProxyStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        this.inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for ProxyStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        this.inner.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.project();
        this.inner.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.project();
        this.inner.poll_flush(cx)
    }
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.project();
        this.inner.poll_write_vectored(cx, bufs)
    }
}
