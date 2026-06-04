use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::collections::BTreeMap;

pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn sign_rpc(
    method: &str,
    params: &BTreeMap<String, String>,
    access_key_secret: &str,
) -> Result<String> {
    let canonical = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let string_to_sign = format!(
        "{method}&{}&{}",
        percent_encode("/"),
        percent_encode(&canonical)
    );
    let key = format!("{access_key_secret}&");
    let mut mac = Hmac::<Sha1>::new_from_slice(key.as_bytes())?;
    mac.update(string_to_sign.as_bytes());
    Ok(B64.encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_space() {
        assert_eq!(percent_encode("a b"), "a%20b");
    }
}
