use std::{fmt, collections::HashMap, io};
use lazy_static::lazy_static;
use anyhow::anyhow;
use md5::{Digest, Md5};
use ring::aead::{UnboundKey, Algorithm, self};
use sha1::Sha1;

fn password_to_cipher_key(password: &str, cipher_len: usize) -> io::Result<Vec<u8>> {
    let pass_bytes = password.as_bytes();
    fn calc(data: &[u8]) -> Vec<u8> {
        let mut hasher = Md5::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    let mut last_digest = calc(pass_bytes);
    let mut key = Vec::clone(&last_digest);
    while (key.len() < cipher_len) {
        last_digest = calc(&[last_digest, pass_bytes.to_vec()].concat());
        key.extend_from_slice(&*&last_digest);
    }
    Ok(key)
}
pub struct CipherInfo {
    pub name: String,
    pub salt_len: usize,
    pub key_len: usize,
    pub nonce_len: usize,
    pub tag_len: usize,
    pub algorithm: &'static Algorithm
}
impl CipherInfo {
    pub fn new(
        name: &str,
        key_len: usize,
        salt_len: usize,
        nonce_len: usize,
        tag_len: usize,
        algorithm: &'static Algorithm,
    ) -> Self {
        Self {
            name: name.to_string(),
            key_len,
            salt_len,
            nonce_len,
            tag_len,
            algorithm
        }
    }
}
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Method {
    AES_192_GCM,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

lazy_static! {
    pub static ref INFOS: HashMap<Method, CipherInfo> = {
        let mut m = HashMap::new();
        m.insert(
            Method::AES_192_GCM,
            CipherInfo::new("aes-128-gcm", 16, 16, 12, 16, &aead::AES_128_GCM),
        );
        m
    };
}


struct NonceSequence {
    v: Vec<u8>
}

impl NonceSequence {
    pub fn new(len: usize) -> Self {
        NonceSequence { v: vec![0xff; len] }
    }

    // https://github.com/v2fly/v2ray-core/blob/3ef7feaeaf737d05c5a624c580633b7ce0f0f1be/common/crypto/auth.go#L43
    pub fn increase(&mut self) -> &[u8]{
        for x in &mut self.v {
            *x = x.wrapping_add(1);
            if *x != 0 {
                break;
            }
        }
        &*self.v
    }
}

fn hkdf(strong_passwd: &[u8],salt: &[u8], info: &[u8], len: usize) -> io::Result<Vec<u8>> {
    let (_, hkdf_struct) = hkdf::Hkdf::<Sha1>::extract(Some(salt), strong_passwd);
    let mut v = vec![0u8; len];
    hkdf_struct.expand(info, &mut *v).map_err(|x| anyhow!("hkdf expand failed"));
    Ok(v)
}

struct Cipher{
    nonce: NonceSequence
}

impl Cipher {
    pub fn new(user_password: &str, salt: &[u8], method: Method) -> anyhow::Result<Cipher>{
        let m = INFOS.get(&method).unwrap();
        // generate strong password
        let strong_password = password_to_cipher_key(user_password, m.key_len)?;
        let psk = hkdf(&strong_password,salt,String::from("ss-subkey").as_bytes(), m.key_len)?;
        let key = UnboundKey::new(m.algorithm,psk.as_ref()).map_err(|_| anyhow!("unboundKey failed"))?;
        Ok(Cipher {
            nonce: NonceSequence::new(m.key_len),
        })
        // let encryptor = 
    }
    pub fn encryptor() {

    }
}