use std::{error::Error, io};

use tokio::io::AsyncReadExt;
use tun::{AsyncDevice, Device, Layer};

pub struct Tun {
    device: AsyncDevice
}


impl Tun {
    pub fn new() -> io::Result<Tun> {
        let mut config = tun::Configuration::default();
        config.address("10.0.0.1").netmask("255.255.255.0").layer(Layer::L3).up();
        let device = match tun::create_as_async(&config) {
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            Ok(x) => x
        };
        Ok(Tun {
            device
        })
    }
    pub async fn run(mut self) -> io::Result<()> {
        let mtu = self.device.get_mut().mtu().expect("mtu");
        let mut packet = vec![0u8; mtu as usize].into_boxed_slice();
        // self.device.read(buf)
        loop {
            tokio::select! {
                n = self.device.read(&mut packet) => {
                    let n = n?;
                    println!("{} bytes read", n);
                }
            }
        }
    }
}