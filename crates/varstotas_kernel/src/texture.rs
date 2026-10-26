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
    /// Colour adjustments, cut-out, decal and world mapping (non-destructive).
    #[serde(default, skip_serializing_if = "TexLook::is_default")]
    pub look: TexLook,
}

/// How a texture image is shown: the image bytes stay untouched.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TexLook {
    /// Hue shift in degrees (-180..180).
    pub hue: f32,
    /// Saturation change (-1..1; -1 = grey).
    pub saturation: f32,
    /// Lightness change (-1..1).
    pub lightness: f32,
    /// Tint the image with the material colour (keeping its brightness).
    pub colorize: bool,
    /// Texels with less alpha are cut out (0 = none).
    pub alpha_cutoff: f32,
    /// One tile, not repeated: off it the material colour shows.
    pub decal: bool,
    /// Coordinates from world position (box projection), ignoring face placements.
    pub world: bool,
}

impl Default for TexLook {
    fn default() -> Self {
        Self { hue: 0.0, saturation: 0.0, lightness: 0.0, colorize: false, alpha_cutoff: 0.5, decal: false, world: false }
    }
}

impl TexLook {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// The colour changes leave pixels as they are.
    pub fn is_identity(&self) -> bool {
        self.hue == 0.0 && self.saturation == 0.0 && self.lightness == 0.0 && !self.colorize
    }

    /// Applies hue / saturation / lightness and colorize (`tint`: the material colour) to RGBA bytes.
    pub fn apply(&self, rgba: &mut [u8], tint: [f32; 3]) {
        if self.is_identity() {
            return;
        }
        let (th, ts, _) = rgb_to_hsl(tint);
        for px in rgba.as_chunks_mut::<4>().0 {
            let c = [px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0];
            let (mut h, mut s, mut l) = rgb_to_hsl(c);
            if self.colorize {
                (h, s) = (th, ts);
            }
            h = (h + self.hue / 360.0).rem_euclid(1.0);
            s = if self.saturation >= 0.0 { s + (1.0 - s) * self.saturation } else { s * (1.0 + self.saturation) };
            l = if self.lightness >= 0.0 { l + (1.0 - l) * self.lightness } else { l * (1.0 + self.lightness) };
            let o = hsl_to_rgb(h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
            for i in 0..3 {
                px[i] = (o[i] * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// RGB (0..1) to hue (0..1), saturation, lightness.
pub fn rgb_to_hsl(c: [f32; 3]) -> (f32, f32, f32) {
    let max = c[0].max(c[1]).max(c[2]);
    let min = c[0].min(c[1]).min(c[2]);
    let l = f32::midpoint(max, min);
    let d = max - min;
    if d < 1e-6 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == c[0] {
        (c[1] - c[2]) / d + if c[1] < c[2] { 6.0 } else { 0.0 }
    } else if max == c[1] {
        (c[2] - c[0]) / d + 2.0
    } else {
        (c[0] - c[1]) / d + 4.0
    };
    (h / 6.0, s, l)
}

/// Hue (0..1), saturation, lightness to RGB (0..1).
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    if s <= 0.0 {
        return [l; 3];
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let f = |t: f32| {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [f(h + 1.0 / 3.0), f(h), f(h - 1.0 / 3.0)]
}

impl TextureImage {
    /// A texture `width_mm` wide; the height follows the image's aspect ratio.
    pub fn new(bytes: Vec<u8>, file_name: impl Into<String>, width_mm: f64) -> Self {
        let pixels = image_dimensions(&bytes).unwrap_or([0, 0]);
        let aspect = if pixels[0] > 0 && pixels[1] > 0 { pixels[1] as f64 / pixels[0] as f64 } else { 1.0 };
        Self { bytes: bytes.into(), pixels, size: [width_mm, width_mm * aspect], file_name: file_name.into(), look: TexLook::default() }
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
    /// Shear after rotating into texture axes, before scaling V (imported atlas mapping).
    pub shear: f64,
}

impl Default for TexPlacement {
    fn default() -> Self {
        Self { offset: [0.0; 2], angle_deg: 0.0, scale: [1.0; 2], shear: 0.0 }
    }
}

/// Scale of a placement axis along which the texture does not change ([`TexPlacement::fit`]).
const FLAT: f64 = 1e6;

impl TexPlacement {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Close enough to be the same mapping (merging coplanar faces).
    pub fn approx_eq(&self, o: &Self) -> bool {
        let close = |a: f64, b: f64, tol: f64| (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs()));
        (0..2).all(|i| close(self.offset[i], o.offset[i], 1e-6) && close(self.scale[i], o.scale[i], 1e-6))
            && close(self.angle_deg, o.angle_deg, 1e-6) && close(self.shear, o.shear, 1e-6)
    }

    /// Fit the placement that maps the planar coordinates of `pts` (on a face with normal `n`)
    /// to the texture coordinates `uvs` (in tiles of `tile` mm), e.g. `vt` from an OBJ file.
    /// Full affine least squares, including shear. `None` for degenerate input.
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
        // The texture doesn't change along an axis (palette models: one texel per triangle, or a
        // stripe of a gradient): that axis gets a huge scale, centred on the points' mean.
        let (l0, l1) = (row0.length(), row1.length());
        let lmax = l0.max(l1);
        if lmax < 1e-9 || a.determinant().abs() < 1e-9 * lmax * lmax {
            let p = if lmax < 1e-9 {
                Self { offset: [0.0; 2], angle_deg: 0.0, scale: [FLAT; 2], shear: 0.0 }
            } else if l0 >= l1 {
                Self { offset: [0.0; 2], angle_deg: row0.y.atan2(row0.x).to_degrees(), scale: [1.0 / l0, FLAT], shear: 0.0 }
            } else {
                Self { offset: [0.0; 2], angle_deg: (-row1.x).atan2(row1.y).to_degrees(), scale: [FLAT, 1.0 / l1], shear: 0.0 }
            };
            let offset = qm - p.to_planar(tm);
            let p = Self { offset: offset.to_array(), ..p };
            return p.is_finite().then_some(p);
        }
        let theta = row0.y.atan2(row0.x);
        let (s, co) = theta.sin_cos();
        let inv_sy = row1.dot(DVec2::new(-s, co));
        if row0.length() < 1e-12 || inv_sy.abs() < 1e-12 || a.determinant().abs() < 1e-18 {
            return None;
        }
        let offset = -(a.inverse() * c);
        let p = Self { offset: offset.to_array(), angle_deg: theta.to_degrees(), scale: [1.0 / row0.length(), 1.0 / inv_sy], shear: row1.dot(DVec2::new(co, s)) / inv_sy };
        p.is_finite().then_some(p)
    }

    fn is_finite(&self) -> bool {
        self.offset.iter().chain(&self.scale).all(|v| v.is_finite()) && self.angle_deg.is_finite() && self.shear.is_finite()
    }

    /// Inverse of [`Self::apply`]: planar coordinates (mm) of texture-space point `t` (mm).
    pub fn to_planar(&self, t: DVec2) -> DVec2 {
        let x = t.x * self.scale[0];
        let st = DVec2::new(x, t.y * self.scale[1] - self.shear * x);
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
        DVec2::new(r.x / sx, (r.y + self.shear * r.x) / sy)
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

/// The placement that keeps a face's texture on its geometry after the face's points moved
/// from `old` (normal `old_n`, placement `old_uv`) to `new` (normal `new_n`), point for point:
/// the texture follows rigid moves, rotations, mirrors and scaling exactly and stretches with
/// the face when only some of its points move (least squares). `None` = the default
/// projection; the old placement is kept when the new points are degenerate.
pub fn carried_placement(old: &[DVec3], old_n: DVec3, old_uv: Option<&TexPlacement>, new: &[DVec3], new_n: DVec3) -> Option<TexPlacement> {
    let mapped: Vec<DVec2> = old.iter().map(|p| face_uv_mm(*p, old_n, old_uv)).collect();
    match TexPlacement::fit(new, &mapped, new_n, [1.0, 1.0]) {
        Some(p) if p.is_identity() || p.approx_eq(&TexPlacement::default()) => None,
        Some(p) => Some(p),
        None => old_uv.copied(),
    }
}

/// One-click changes of a face's texture placement ([`placement_op`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TexOp {
    /// One tile exactly covers the face (in the texture's current directions).
    Fit,
    /// The texture turns 90° on the face, about the face's centre.
    Rotate90,
    /// Mirror left ↔ right about the face's centre.
    FlipU,
    /// Mirror top ↔ bottom about the face's centre.
    FlipV,
}

/// The placement after `op` on a face with points `pts` (local), normal `n`, placement `uv`
/// and tile size `tile` (mm). Outer `None`: degenerate face; inner `None`: the default.
pub fn placement_op(op: TexOp, pts: &[DVec3], n: DVec3, uv: Option<&TexPlacement>, tile: [f64; 2]) -> Option<Option<TexPlacement>> {
    let t: Vec<DVec2> = pts.iter().map(|p| face_uv_mm(*p, n, uv)).collect();
    let lo = t.iter().copied().reduce(DVec2::min)?;
    let hi = t.iter().copied().reduce(DVec2::max)?;
    let c = (lo + hi) / 2.0;
    let size = hi - lo;
    let want: Vec<DVec2> = match op {
        TexOp::Fit => {
            if size.x < 1e-9 || size.y < 1e-9 {
                return None;
            }
            t.iter().map(|q| (*q - lo) / size * DVec2::from(tile)).collect()
        }
        TexOp::Rotate90 => t.iter().map(|q| c + DVec2::new(q.y - c.y, c.x - q.x)).collect(),
        TexOp::FlipU => t.iter().map(|q| DVec2::new(2.0 * c.x - q.x, q.y)).collect(),
        TexOp::FlipV => t.iter().map(|q| DVec2::new(q.x, 2.0 * c.y - q.y)).collect(),
    };
    let p = TexPlacement::fit(pts, &want, n, [1.0, 1.0])?;
    Some((!(p.is_identity() || p.approx_eq(&TexPlacement::default()))).then_some(p))
}

/// The placement whose texture `u` axis runs along the edge `a` → `b` (local points on a
/// face with normal `n`), its origin at `a`; the tile size (scale) of `uv` is kept.
pub fn align_to_edge(n: DVec3, uv: Option<&TexPlacement>, a: DVec3, b: DVec3) -> TexPlacement {
    let (qa, qb) = (face_uv_mm(a, n, None), face_uv_mm(b, n, None));
    let d = qb - qa;
    let scale = uv.map_or([1.0, 1.0], |p| p.scale);
    TexPlacement { offset: qa.to_array(), angle_deg: d.y.atan2(d.x).to_degrees(), scale, shear: 0.0 }
}

/// The placement of face `dst` that continues the texture of face `src` (placement
/// `src_uv`): across a shared edge the face is unfolded about that edge into `src`'s plane
/// (the texture wraps round the corner without a seam); otherwise `dst` is projected onto
/// `src`'s plane. Outer `None`: no sensible continuation (perpendicular, degenerate).
pub fn continue_placement(mesh: &crate::Mesh, src: crate::FaceId, src_uv: Option<&TexPlacement>, dst: crate::FaceId) -> Option<Option<TexPlacement>> {
    let (sf, df) = (mesh.faces.get(src)?, mesh.faces.get(dst)?);
    let pts: Vec<DVec3> = mesh.face_vertices(dst).iter().filter_map(|v| mesh.vertices.get(*v).map(|x| x.pos)).collect();
    let (ns, nd) = (sf.normal, df.normal);
    let src_edges = mesh.face_edges(src);
    let shared = mesh.face_edges(dst).into_iter().find(|e| src_edges.contains(e)).and_then(|e| mesh.edges.get(e));
    let flat: Vec<DVec3> = match shared {
        Some(e) => {
            let (a, b) = (mesh.vertices[e.v[0]].pos, mesh.vertices[e.v[1]].pos);
            let axis = (b - a).normalize_or_zero();
            let rot = if nd.dot(ns) < -0.9999 {
                glam::DQuat::from_axis_angle(axis, std::f64::consts::PI)
            } else {
                glam::DQuat::from_rotation_arc(nd, ns)
            };
            pts.iter().map(|p| a + rot * (*p - a)).collect()
        }
        None => {
            if nd.dot(ns).abs() < 0.05 {
                return None;
            }
            let o = mesh.vertices.get(*sf.outer.first()?)?.pos;
            pts.iter().map(|p| *p - ns * (*p - o).dot(ns)).collect()
        }
    };
    let mapped: Vec<DVec2> = flat.iter().map(|p| face_uv_mm(*p, ns, src_uv)).collect();
    let p = TexPlacement::fit(&pts, &mapped, nd, [1.0, 1.0])?;
    Some((!(p.is_identity() || p.approx_eq(&TexPlacement::default()))).then_some(p))
}

/// Placements of `faces` that continue the texture of `root` (placement `root_uv`): faces
/// reached through shared edges are unfolded one after another (breadth first), the rest are
/// projected from `root`. The root itself is not in the result.
pub fn wrap_placements(mesh: &crate::Mesh, faces: &[crate::FaceId], root: crate::FaceId, root_uv: Option<TexPlacement>) -> Vec<(crate::FaceId, Option<TexPlacement>)> {
    let mut placed: Vec<(crate::FaceId, Option<TexPlacement>)> = vec![(root, root_uv)];
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(f) = queue.pop_front() {
        let uv = placed.iter().find(|(g, _)| *g == f).and_then(|x| x.1);
        for e in mesh.face_edges(f) {
            for g in mesh.edge_faces(e) {
                if !faces.contains(&g) || placed.iter().any(|(h, _)| *h == g) {
                    continue;
                }
                if let Some(p) = continue_placement(mesh, f, uv.as_ref(), g) {
                    placed.push((g, p));
                    queue.push_back(g);
                }
            }
        }
    }
    for g in faces {
        if !placed.iter().any(|(h, _)| h == g)
            && let Some(p) = continue_placement(mesh, root, root_uv.as_ref(), *g)
        {
            placed.push((*g, p));
        }
    }
    placed.remove(0);
    placed
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
    base64_decode(&s).map(Into::into).ok_or_else(|| serde::de::Error::custom("bad base64 texture")) // i18n-ignore
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
            TexPlacement { offset: [120.0, -40.0], angle_deg: 30.0, scale: [1.5, 0.75], shear: 0.0 },
            TexPlacement { offset: [5.0, 6.0], angle_deg: -100.0, scale: [2.0, -1.0], shear: 0.0 },
            TexPlacement { offset: [50.0, -70.0], angle_deg: 43.0, scale: [0.7, -2.0], shear: 1.3 },
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
    fn fit_palette_and_stripe_uvs() {
        // Kenney-style palette models: every corner of a triangle on one texel, or a gradient
        // stripe (u fixed, v varying). The fit keeps those texture coordinates on the face.
        let n = DVec3::new(0.2, -1.0, 0.4).normalize();
        let (u, v) = texture_axes(n);
        let pts: Vec<DVec3> =
            [(0.0, 0.0), (400.0, 30.0), (100.0, 300.0)].iter().map(|(a, b)| DVec3::new(5e3, -2e3, 700.0) + u * *a + v * *b).collect();
        let tile = [1000.0, 1000.0];
        for uvs in [vec![DVec2::new(0.71, -0.62); 3], vec![DVec2::new(0.3, -0.1), DVec2::new(0.3, -0.2), DVec2::new(0.3, -0.15)]] {
            let got = TexPlacement::fit(&pts, &uvs, n, tile).expect("a placement");
            for (p, want) in pts.iter().zip(&uvs) {
                let t = face_uv_mm(*p, n, Some(&got)) / DVec2::from(tile);
                assert!((t - *want).length() < 1e-4, "{t} vs {want} ({got:?})");
            }
        }
    }

    #[test]
    fn placement_moves_rotates_scales() {
        let p = TexPlacement { offset: [100.0, 0.0], angle_deg: 90.0, scale: [2.0, 1.0], shear: 0.0 };
        let q = p.apply(DVec2::new(100.0, 50.0));
        assert!((q - DVec2::new(25.0, 0.0)).length() < 1e-9, "{q}");
        assert!((p.to_planar(q) - DVec2::new(100.0, 50.0)).length() < 1e-9);
    }
}
