use std::{
    io::{self},
    process::Command,
};

pub fn get_default_ipv4_gateway() -> io::Result<String> {
    let output = Command::new("ip")
        .arg("route")
        .arg("show")
        .output()
        .expect("default gateway error");
    let stdout = &*output.stdout;
    let out = String::from_utf8_lossy(stdout).to_string();
    let default = out
        .lines()
        .filter(|x| x.contains("via"))
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<&str>>()[2]
        .to_string();
    Ok(default)
}

#[test]
pub fn test_defaultipv4() {
    let r = get_default_ipv4_gateway();
    assert!(r.is_ok());
}