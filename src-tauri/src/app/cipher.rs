use anyhow::{Context, Result};

#[cfg(windows)]
pub fn protect(plain: &[u8]) -> Result<Vec<u8>> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};

    let mut buf = plain.to_vec();
    let mut data_in = CRYPT_INTEGER_BLOB {
        cbData: buf.len() as u32,
        pbData: buf.as_mut_ptr(),
    };
    let mut data_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(&mut data_in, None, None, None, None, 0, &mut data_out)
            .context("本机加密失败")?;
        let enc = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(data_out.pbData as _));
        Ok(enc)
    }
}

#[cfg(windows)]
pub fn unprotect(enc: &[u8]) -> Result<Vec<u8>> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    let mut data_in = CRYPT_INTEGER_BLOB {
        cbData: enc.len() as u32,
        pbData: enc.as_ptr() as *mut u8,
    };
    let mut data_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(&mut data_in, None, None, None, None, 0, &mut data_out)
            .context("本机解密失败，请确认文件由本机导出且未被篡改")?;
        let plain = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(data_out.pbData as _));
        Ok(plain)
    }
}

#[cfg(not(windows))]
pub fn protect(_: &[u8]) -> Result<Vec<u8>> {
    anyhow::bail!("仅支持 Windows")
}

#[cfg(not(windows))]
pub fn unprotect(_: &[u8]) -> Result<Vec<u8>> {
    anyhow::bail!("仅支持 Windows")
}
