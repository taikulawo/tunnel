use std::os::unix::prelude::AsRawFd;

use tokio::net::TcpSocket;

pub fn bind_to_device(socket: &TcpSocket, interface_name: &str) {
    unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_BINDTODEVICE,
            interface_name.as_ptr() as *const _ as *const _,
            interface_name.len() as _,
        )
    };
}
