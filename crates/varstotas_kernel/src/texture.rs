//! Texture images on materials and texture coordinates of faces.
//!
//! A textured material embeds the encoded image file (PNG or JPEG; base64 in `.vrst`) and
//! the real-world size of one tile. Faces get planar coordinates in millimetres from
//! [`texture_axes`] (horizontal faces: red/green; other faces: horizontal u, upward v),
//! optionally moved by a per-face [`TexPlacement`]; dividing by the tile size gives UVs.

use glam::{DVec2, DVec3};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

/// An image attached to a material. The bytes are shared (`Arc`), so undo snapshots don't copy them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureImage {
    /// The encoded image file as loaded (PNG or JPEG).
    #[serde(serialize_with = "ser_b64", deserialize_with = "de_b64")]
    pub bytes: Arc<[u8]>,
    /// Pixel size `[width, height]` (0 when unknown).
    pub pixels: [u32; 2],
    /// Size of one tile in model units (mm): `[width, height]`.
    pub size: [f64; 2],
    /// Original file name (for export and display).
    #[serde(default)]
    pub file_name: String,
}

impl TextureImage {
    /// A texture `width_mm` wide; the height follows the image's aspect ratio.
    pub fn new(bytes: Vec<u8>, file_name: impl Into<String>, width_mm: f64) -> Self {
        let pixels = image_dimensions(&bytes).unwrap_or([0, 0]);
        let aspect = if pixels[0] > 0 && pixels[1] > 0 { pixels[1] as f64 / pixels[0] as f64 } else { 1.0 };
        Self { bytes: bytes.into(), pixels, size: [width_mm, width_mm * aspect], file_name: file_name.into() }
    }

    /// File extension for the embedded format (`png` or `jpg`).
    pub fn extension(&self) -> &'static str {
        if self.bytes.starts_with(b"\x89PNG") { "png" } else { "jpg" }
    }
}

/// Per-face texture position (Texture Position): planar coordinates are shifted by `-offset`,
/// rotated by `-angle_deg` and divided by `scale`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TexPlacement {
    /// Texture origin in the face's planar coordinates, mm.
    pub offset: [f64; 2],
    pub angle_deg: f64,
    pub scale: [f64; 2],
}

impl Default for TexPlacement {
    fn default() -> Self {
        Self { offset: [0.0; 2], angle_deg: 0.0, scale: [1.0; 2] }
    }
}

impl TexPlacement {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Close enough to be the same mapping (merging coplanar faces).
    pub fn approx_eq(&self, o: &Self) -> bool {
        let close = |a: f64, b: f64, tol: f64| (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs()));
        (0..2).all(|i| close(self.offset[i], o.offset[i], 1e-6) && close(self.scale[i], o.scale[i], 1e-6))
            && close(self.angle_deg, o.angle_deg, 1e-6)
    }

    /// Fit the placement that maps the planar coordinates of `pts` (on a face with normal `n`)
    /// to the texture coordinates `uvs` (in tiles of `tile` mm), e.g. `vt` from an OBJ file.
    /// Least squares over all points; shear is dropped. `None` for degenerate input.
    pub fn fit(pts: &[DVec3], uvs: &[DVec2], n: DVec3, tile: [f64; 2]) -> Option<Self> {
        if pts.len() < 3 || pts.len() != uvs.len() {
            return None;
        }
        let q: Vec<DVec2> = pts.iter().map(|p| face_uv_mm(*p, n, None)).collect();
        let t: Vec<DVec2> = uvs.iter().map(|u| DVec2::new(u.x * tile[0], u.y * tile[1])).collect();
        let k = q.len() as f64;
        let (qm, tm) = (q.iter().sum::<DVec2>() / k, t.iter().sum::<DVec2>() / k);
        // A = T Q⁻¹ with Q = Σ dq dqᵀ, T = Σ dt dqᵀ (columns of glam matrices).
        let (mut qq, mut tq) = (glam::DMat2::ZERO, glam::DMat2::ZERO);
        for (a, b) in q.iter().zip(&t) {
            let (dq, dt) = (*a - qm, *b - tm);
            qq += glam::DMat2::from_cols(dq * dq.x, dq * dq.y);
            tq += glam::DMat2::from_cols(dt * dq.x, dt * dq.y);
        }
        let tr = qq.col(0).x + qq.col(1).y;
        if tr <= 0.0 || qq.determinant() < 1e-10 * tr * tr {
            return None;
        }
        let a = tq * qq.inverse();
        let c = tm - a * qm;
        // a = diag(1/sx, 1/sy) · R(-θ); rows: (cos, sin)/sx and (-sin, cos)/sy.
        let row0 = DVec2::new(a.col(0).x, a.col(1).x);
        let row1 = DVec2::new(a.col(0).y, a.col(1).y);
        let theta = row0.y.atan2(row0.x);
        let (s, co) = theta.sin_cos();
        let inv_sy = row1.dot(DVec2::new(-s, co));
        if row0.length() < 1e-12 || inv_sy.abs() < 1e-12 || a.determinant().abs() < 1e-18 {
            return None;
        }
        let offset = -(a.inverse() * c);
        let p = Self { offset: offset.to_array(), angle_deg: theta.to_degrees(), scale: [1.0 / row0.length(), 1.0 / inv_sy] };
        p.is_finite().then_some(p)
    }

    fn is_finite(&self) -> bool {
        self.offset.iter().chain(&self.scale).all(|v| v.is_finite()) && self.angle_deg.is_finite()
    }

    /// Inverse of [`Self::apply`]: planar coordinates (mm) of texture-space point `t` (mm).
    pub fn to_planar(&self, t: DVec2) -> DVec2 {
        let st = DVec2::new(t.x * self.scale[0], t.y * self.scale[1]);
        let (s, c) = self.angle_deg.to_radians().sin_cos();
        DVec2::from(self.offset) + DVec2::new(st.x * c - st.y * s, st.x * s + st.y * c)
    }

    /// Apply to planar coordinates (mm).
    pub fn apply(&self, q: DVec2) -> DVec2 {
        let d = q - DVec2::from(self.offset);
        let (s, c) = (-self.angle_deg.to_radians()).sin_cos();
        let r = DVec2::new(d.x * c - d.y * s, d.x * s + d.y * c);
        let sx = if self.scale[0].abs() > 1e-9 { self.scale[0] } else { 1.0 };
        let sy = if self.scale[1].abs() > 1e-9 { self.scale[1] } else { 1.0 };
        DVec2::new(r.x / sx, r.y / sy)
    }
}

/// Planar texture axes `(u, v)` of a face with unit normal `n`: horizontal faces use red/green
/// (mirrored on downward faces so the image reads correctly from below), others a horizontal
/// `u` (to the right when facing the front) and an upward `v`.
pub fn texture_axes(n: DVec3) -> (DVec3, DVec3) {
    if n.z.abs() > 0.999 {
        return (DVec3::X * n.z.signum(), DVec3::Y);
    }
    let u = DVec3::Z.cross(n).normalize_or(DVec3::X);
    (u, n.cross(u))
}

/// Planar texture coordinates (mm) of local point `p` on a face with normal `n`.
pub fn face_uv_mm(p: DVec3, n: DVec3, place: Option<&TexPlacement>) -> DVec2 {
    let (u, v) = texture_axes(n);
    let q = DVec2::new(p.dot(u), p.dot(v));
    place.map_or(q, |pl| pl.apply(q))
}

/// Pixel size of a PNG or JPEG file from its header.
pub fn image_dimensions(b: &[u8]) -> Option<[u32; 2]> {
    if b.len() >= 24 && b.starts_with(b"\x89PNG\r\n\x1a\n") && &b[12..16] == b"IHDR" {
        let be = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        return Some([be(16), be(20)]);
    }
    if b.len() >= 4 && b[0] == 0xFF && b[1] == 0xD8 {
        let mut i = 2;
        while i + 9 < b.len() {
            if b[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = b[i + 1];
            if marker == 0xFF || marker == 0x01 || (0xD0..=0xD9).contains(&marker) {
                i += 2;
                continue;
            }
            let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
            // SOF0..SOF15 except DHT (C4), JPG (C8) and DAC (CC).
            if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
                let h = u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32;
                let w = u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32;
                return Some([w, h]);
            }
            i += 2 + len;
        }
    }
    None
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= c.len() {
                out.push(B64[(n >> (18 - 6 * k) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut val = [255u8; 256];
    for (i, c) in B64.iter().enumerate() {
        val[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=') {
        let v = val[c as usize];
        if v == 255 {
            return None;
        }
        acc = acc << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

fn ser_b64<S: Serializer>(b: &Arc<[u8]>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&base64_encode(b))
}

fn de_b64<'de, D: Deserializer<'de>>(d: D) -> Result<Arc<[u8]>, D::Error> {
    let s = String::deserialize(d)?;
    base64_decode(&s).map(Into::into).ok_or_else(|| serde::de::Error::custom("bad base64 texture"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip() {
        for n in 0..10 {
            let data: Vec<u8> = (0..n).map(|i| (i * 37 + 5) as u8).collect();
            assert_eq!(base64_decode(&base64_encode(&data)).unwrap(), data);
        }
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
    }

    #[test]
    fn axes_are_right_handed_in_plane() {
        for n in [DVec3::X, -DVec3::Y, DVec3::Z, -DVec3::Z, DVec3::new(1.0, 1.0, 1.0).normalize()] {
            let (u, v) = texture_axes(n);
            assert!(u.dot(n).abs() < 1e-9 && v.dot(n).abs() < 1e-9);
            assert!((u.cross(v).dot(n) - 1.0).abs() < 1e-9, "{n}");
        }
        // Front wall (normal -Y): u = +X, v = +Z.
        let (u, v) = texture_axes(-DVec3::Y);
        assert!((u - DVec3::X).length() < 1e-9 && (v - DVec3::Z).length() < 1e-9);
    }

    #[test]
    fn fit_recovers_placement_incl_mirror() {
        let n = DVec3::new(0.3, -1.0, 0.2).normalize();
        let (u, v) = texture_axes(n);
        let pts: Vec<DVec3> = [(0.0, 0.0), (900.0, 50.0), (300.0, 700.0), (-200.0, 400.0)]
            .iter()
            .map(|(a, b)| DVec3::new(10.0, 20.0, 30.0) + u * *a + v * *b)
            .collect();
        for want in [
            TexPlacement { offset: [120.0, -40.0], angle_deg: 30.0, scale: [1.5, 0.75] },
            TexPlacement { offset: [5.0, 6.0], angle_deg: -100.0, scale: [2.0, -1.0] },
        ] {
            let tile = [400.0, 200.0];
            let uvs: Vec<DVec2> =
                pts.iter().map(|p| face_uv_mm(*p, n, Some(&want)) / DVec2::from(tile)).collect();
            let got = TexPlacement::fit(&pts, &uvs, n, tile).unwrap();
            for p in &pts {
                let (a, b) = (face_uv_mm(*p, n, Some(&got)), face_uv_mm(*p, n, Some(&want)));
                assert!((a - b).length() < 1e-6, "{got:?} vs {want:?}");
            }
        }
        assert!(TexPlacement::fit(&pts[..2], &[DVec2::ZERO; 2], n, [1.0; 2]).is_none());
    }

    #[test]
    fn placement_moves_rotates_scales() {
        let p = TexPlacement { offset: [100.0, 0.0], angle_deg: 90.0, scale: [2.0, 1.0] };
        let q = p.apply(DVec2::new(100.0, 50.0));
        assert!((q - DVec2::new(25.0, 0.0)).length() < 1e-9, "{q}");
        assert!((p.to_planar(q) - DVec2::new(100.0, 50.0)).length() < 1e-9);
    }
}
