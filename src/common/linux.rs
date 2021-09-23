use std::{
    io::{self, BufRead},
    process::Command,
    str::FromStr,
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
