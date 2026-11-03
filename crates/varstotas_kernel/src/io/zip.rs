//! Zip archives: reading them (3MF packages, downloaded model and texture archives) and writing
//! stored ones (3MF packages, the web build's data folder).

use std::io::Read;
use varstotas_i18n::tr;

/// The files of a zip archive whose names pass `want` (stored or deflated; no zip64).
pub fn unzip(data: &[u8], want: impl Fn(&str) -> bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let u16_at = |o: usize| data.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize).ok_or_else(|| tr!("kernel-io-truncated-zip"));
    let u32_at = |o: usize| data.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).ok_or_else(|| tr!("kernel-io-truncated-zip"));
    // End of central directory: the last "PK\5\6" (a comment may follow it).
    let eocd = (0..data.len().saturating_sub(21)).rev().take(65_557).find(|&i| data[i..].starts_with(b"PK\x05\x06")).ok_or("not a zip archive")?;
    let count = u16_at(eocd + 10)?;
    if count > 4096 {
        return Err("ZIP has too many entries".into());
    }
    let mut total = 0usize;
    let mut at = u32_at(eocd + 16)?;
    let mut out = Vec::new();
    for _ in 0..count {
        if !data.get(at..).is_some_and(|d| d.starts_with(b"PK\x01\x02")) {
            return Err(tr!("kernel-io-bad-zip-directory"));
        }
        let method = u16_at(at + 10)?;
        let (csize, usize_) = (u32_at(at + 20)?, u32_at(at + 24)?);
        let (nlen, elen, clen) = (u16_at(at + 28)?, u16_at(at + 30)?, u16_at(at + 32)?);
        if usize_ > 128 * 1024 * 1024 || usize_ > csize.saturating_mul(200).max(1024 * 1024) {
            return Err("ZIP entry exceeds decompression limits".into());
        }
        total = total.checked_add(usize_).ok_or("ZIP size overflow")?;
        if total > 512 * 1024 * 1024 {
            return Err("ZIP exceeds total decompression limit".into());
        }
        let local = u32_at(at + 42)?;
        let name = String::from_utf8_lossy(data.get(at + 46..at + 46 + nlen).ok_or_else(|| tr!("kernel-io-truncated-zip"))?).to_string();
        at += 46 + nlen + elen + clen;
        if name.ends_with('/') || !want(&name) {
            continue;
        }
        if !data.get(local..).is_some_and(|d| d.starts_with(b"PK\x03\x04")) {
            return Err(tr!("kernel-io-bad-zip-entry-name", name = name));
        }
        let start = local + 30 + u16_at(local + 26)? + u16_at(local + 28)?;
        let raw = data.get(start..start + csize).ok_or_else(|| tr!("kernel-io-truncated-zip"))?;
        if method == 0 && csize != usize_ { return Err("ZIP stored size mismatch".into()); }
        let bytes = match method {
            0 => raw.to_vec(),
            8 => {
                let mut v = Vec::new();
                flate2::read::DeflateDecoder::new(raw)
                    .take(usize_ as u64 + 1).read_to_end(&mut v).map_err(|e| format!("{name}: {e}"))?;
                v
            }
            m => return Err(tr!("kernel-io-name-zip-method-m-not-supported", name = name, m = m.to_string())),
        };
        if bytes.len() != usize_ {
            return Err("ZIP decompressed size mismatch".into());
        }
        out.push((name, bytes));
    }
    Ok(out)
}

/// CRC-32 (IEEE) of `data`, as zip entries carry it.
pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut c = !0u32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    !c
}

/// A ZIP archive with stored (uncompressed) entries.
pub fn zip_stored(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let crc = crc32(data);
        let offset = out.len() as u32;
        let header = |sig: u32, central: bool| {
            let mut h = Vec::new();
            h.extend(sig.to_le_bytes());
            if central {
                h.extend(20u16.to_le_bytes()); // made by
            }
            h.extend(20u16.to_le_bytes()); // version needed
            h.extend(0u16.to_le_bytes()); // flags
            h.extend(0u16.to_le_bytes()); // stored
            h.extend(0u16.to_le_bytes()); // time
            h.extend(0x21u16.to_le_bytes()); // date: 1980-01-01
            h.extend(crc.to_le_bytes());
            h.extend((data.len() as u32).to_le_bytes());
            h.extend((data.len() as u32).to_le_bytes());
            h.extend((name.len() as u16).to_le_bytes());
            h.extend(0u16.to_le_bytes()); // extra
            if central {
                h.extend(0u16.to_le_bytes()); // comment
                h.extend(0u16.to_le_bytes()); // disk
                h.extend(0u16.to_le_bytes()); // internal attrs
                h.extend(0u32.to_le_bytes()); // external attrs
                h.extend(offset.to_le_bytes());
            }
            h.extend(name.as_bytes());
            h
        };
        out.extend(header(0x0403_4B50, false));
        out.extend_from_slice(data);
        central.extend(header(0x0201_4B50, true));
    }
    let cd_offset = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend(central);
    out.extend(0x0605_4B50u32.to_le_bytes());
    out.extend([0u8; 4]); // disk numbers
    out.extend((files.len() as u16).to_le_bytes());
    out.extend((files.len() as u16).to_le_bytes());
    out.extend(cd_len.to_le_bytes());
    out.extend(cd_offset.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}
