//! Reading zip archives (3MF packages, downloaded model and texture archives).

use std::io::Read;

/// The files of a zip archive whose names pass `want` (stored or deflated; no zip64).
pub fn unzip(data: &[u8], want: impl Fn(&str) -> bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let u16_at = |o: usize| data.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize).ok_or("truncated zip");
    let u32_at = |o: usize| data.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).ok_or("truncated zip");
    // End of central directory: the last "PK\5\6" (a comment may follow it).
    let eocd = (0..data.len().saturating_sub(21)).rev().take(65_557).find(|&i| data[i..].starts_with(b"PK\x05\x06")).ok_or("not a zip archive")?;
    let count = u16_at(eocd + 10)?;
    let mut at = u32_at(eocd + 16)?;
    let mut out = Vec::new();
    for _ in 0..count {
        if !data.get(at..).is_some_and(|d| d.starts_with(b"PK\x01\x02")) {
            return Err("bad zip directory".into());
        }
        let method = u16_at(at + 10)?;
        let (csize, usize_) = (u32_at(at + 20)?, u32_at(at + 24)?);
        let (nlen, elen, clen) = (u16_at(at + 28)?, u16_at(at + 30)?, u16_at(at + 32)?);
        let local = u32_at(at + 42)?;
        let name = String::from_utf8_lossy(data.get(at + 46..at + 46 + nlen).ok_or("truncated zip")?).to_string();
        at += 46 + nlen + elen + clen;
        if name.ends_with('/') || !want(&name) {
            continue;
        }
        if !data.get(local..).is_some_and(|d| d.starts_with(b"PK\x03\x04")) {
            return Err(format!("bad zip entry {name}"));
        }
        let start = local + 30 + u16_at(local + 26)? + u16_at(local + 28)?;
        let raw = data.get(start..start + csize).ok_or("truncated zip")?;
        let bytes = match method {
            0 => raw.to_vec(),
            8 => {
                let mut v = Vec::with_capacity(usize_);
                flate2::read::DeflateDecoder::new(raw).read_to_end(&mut v).map_err(|e| format!("{name}: {e}"))?;
                v
            }
            m => return Err(format!("{name}: zip method {m} not supported")),
        };
        out.push((name, bytes));
    }
    Ok(out)
}
