use anyhow::{anyhow, Result};
use std::net::Ipv4Addr;

pub fn is_ipv4(s: &str) -> bool {
    s.trim().parse::<Ipv4Addr>().is_ok()
}

const PROBE_RETRIES: u32 = 2;
const RETRY_DELAY_MS: u64 = 400;

pub async fn fetch_public_ip(urls: &[String]) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()?;
    let mut last_err = None;
    for url in urls {
        for attempt in 0..=PROBE_RETRIES {
            match client.get(url).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(r) => {
                        let text = r.text().await?.trim().to_string();
                        if is_ipv4(&text) {
                            return Ok(text);
                        }
                        last_err = Some(anyhow!("invalid ipv4 from {url}: {text}"));
                    }
                    Err(e) => last_err = Some(e.into()),
                },
                Err(e) => last_err = Some(e.into()),
            }
            if attempt < PROBE_RETRIES {
                tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("no probe urls configured")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_ipv4() {
        assert!(is_ipv4("1.2.3.4"));
        assert!(!is_ipv4("not-an-ip"));
        assert!(!is_ipv4("2001:db8::1"));
    }
}
