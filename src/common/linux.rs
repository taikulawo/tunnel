use std::{io::{self, BufRead, ErrorKind}, net::IpAddr, process::Command, str::FromStr};
use anyhow::{
    Result,
    anyhow
};
use log::error;

pub fn get_default_ipv4_gateway() -> Result<IpAddr> {
    let output = Command::new("ip")
        .arg("route")
        .arg("show")
        .arg("table")
        .arg("default")
        .output()
        .expect("get ipv4 default gateway error");
    let stdout = &*output.stdout;
    let out = String::from_utf8_lossy(stdout).to_string();
    let default = out
        .lines()
        .filter(|x| x.contains("via"))
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<&str>>()[2];
    let addr = IpAddr::from_str(default)?;
    Ok(addr)
}

pub fn get_default_ipv6_gateway() -> Result<IpAddr> {
    let output = Command::new("ip")
        .arg("route")
        .arg("show")
        .arg("table")
        .arg("default")
        .output()
        .expect("get ipv6 default gateway error");
    if !output.status.success() {
        return Err(anyhow!("exec failed {}", String::from_utf8_lossy(&*output.stderr)));
    }
    let line = String::from_utf8_lossy(&*output.stdout);
    let line = line.lines().filter(|s| s.contains("default"))
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<&str>>()[2];
    let addr = IpAddr::from_str(line)?;
    Ok(addr)
}

pub fn get_default_interface() -> io::Result<String> {
    let output = Command::new("ip").arg("route").arg("show").output()?;
    let out = String::from_utf8_lossy(&*output.stdout).to_string();
    let line = out
        .lines()
        .filter(|s| s.contains("default"))
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<&str>>();
    let a = line.last().unwrap();
    Ok(String::from(*a))
}

#[test]
pub fn test_defaultipv4() {
    let r = get_default_ipv4_gateway();
    assert!(r.is_ok());
}
