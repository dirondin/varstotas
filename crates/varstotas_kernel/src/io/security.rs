//! Shared limits and path confinement for untrusted model files.
use super::IoError;
use std::io::Read;
use std::path::{Path, PathBuf};
pub const MAX_MODEL_BYTES: usize = 256 * 1024 * 1024;

pub fn read_limited(path: &Path, max: usize) -> std::io::Result<Vec<u8>> {
    // Reject FIFOs/devices before open, which could itself block.
    if !std::fs::metadata(path)?.is_file() { return Err(std::io::Error::other("Expected a regular file")); }
    let file = std::fs::File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > max as u64 {
        return Err(std::io::Error::other(
            "File is not regular or exceeds the size limit",
        ));
    }
    let mut bytes = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(std::io::Error::other("File exceeds the size limit"));
    }
    Ok(bytes)
}

pub fn resource_path(root: &Path, name: &str) -> std::io::Result<PathBuf> {
    let name = name.replace('\\', "/");
    if name.starts_with('/') || name.contains(':') || name.split('/').any(|s| s == "..") {
        return Err(std::io::Error::other(
            "External resource escapes the model directory",
        ));
    }
    let root = root.canonicalize()?;
    let path = root.join(name).canonicalize()?;
    if !path.starts_with(&root) {
        return Err(std::io::Error::other(
            "External resource escapes the model directory",
        ));
    }
    Ok(path)
}

/// Reject invalid references and cycles before caches or recursive scene traversal run.
pub fn validate_model(m: &crate::Model) -> Result<(), IoError> {
    use std::collections::HashMap;
    let bad = || IoError::Format("Invalid, cyclic, or oversized model graph".into());
    if m.defs.len() > 50_000 {
        return Err(bad());
    }
    let mut stored = 0usize;
    for (_, d) in &m.defs {
        let mesh = &d.mesh;
        stored = stored.saturating_add(
            mesh.vertices.len() + mesh.edges.len() + mesh.faces.len() + mesh.instances.len(),
        );
        if stored > 2_000_000 {
            return Err(bad());
        }
        if mesh.vertices.values().any(|v| !v.pos.is_finite()) {
            return Err(bad());
        }
        if mesh
            .edges
            .values()
            .any(|e| e.v.iter().any(|v| !mesh.vertices.contains_key(*v)))
        {
            return Err(bad());
        }
        for f in mesh.faces.values() {
            if !f.normals.is_empty() && (f.normals.len() != f.loops().map(Vec::len).sum::<usize>()
                || f.normals.iter().any(|n| !n.is_finite() || !n.length_squared().is_finite() || n.length_squared() < 1e-12)) {
                return Err(IoError::Format(format!("Invalid face shading normals: {} corners, {} normals", f.loops().map(Vec::len).sum::<usize>(), f.normals.len())));
            }
            if f.uv.is_some_and(|uv| !uv.offset.iter().chain(&uv.scale).all(|v| v.is_finite())
                || !uv.angle_deg.is_finite() || !uv.shear.is_finite()) { return Err(bad()); }
            if !f.normal.is_finite() || f.outer.len() < 3 {
                return Err(bad());
            }
            if std::iter::once(&f.outer)
                .chain(&f.holes)
                .any(|l| l.len() < 3 || l.iter().any(|v| !mesh.vertices.contains_key(*v)))
            {
                return Err(bad());
            }
            if [f.front, f.back]
                .into_iter()
                .flatten()
                .any(|id| !m.materials.contains_key(id))
            {
                return Err(bad());
            }
        }
        if mesh
            .instances
            .values()
            .any(|i| !m.defs.contains_key(i.def) || !i.transform.is_finite())
        {
            return Err(bad());
        }
    }
    fn visit(
        m: &crate::Model,
        id: crate::DefId,
        depth: usize,
        state: &mut HashMap<crate::DefId, (bool, usize, usize)>,
    ) -> Result<usize, IoError> {
        let bad = || IoError::Format("Invalid, cyclic, or oversized model graph".into());
        if depth > 64 {
            return Err(bad());
        }
        if let Some((done, size, height)) = state.get(&id) {
            return if *done && depth + height <= 64 {
                Ok(*size)
            } else {
                Err(bad())
            };
        }
        state.insert(id, (false, 0, 0));
        let mut height = 0;
        let d = m.defs.get(id).ok_or_else(bad)?;
        let mut size = 1 + d.mesh.vertices.len() + d.mesh.edges.len() + d.mesh.faces.len();
        for i in d.mesh.instances.values() {
            size = size
                .checked_add(visit(m, i.def, depth + 1, state)?)
                .ok_or_else(bad)?;
            height = height.max(1 + state[&i.def].2);
            if size > 2_000_000 || depth + height > 64 {
                return Err(bad());
            }
        }
        state.insert(id, (true, size, height));
        Ok(size)
    }
    let mut state = HashMap::new();
    visit(m, m.root, 0, &mut state)?;
    for id in m.defs.keys() {
        visit(m, id, 0, &mut state)?;
    }
    Ok(())
}

/// Replace a file without following a destination symlink or a predictable temp symlink.
/// Parent directories must be trusted by the caller.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(std::io::Error::other(
            "Refusing to overwrite a symbolic link",
        ));
    }
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let (tmp, mut file) = loop {
        let tmp = dir.join(format!(
            ".varstotas-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&tmp) {
            Ok(file) => break (tmp, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

/// Every output of a multi-file export, calculated before any file is changed.
pub fn export_bundle(model: &crate::Model, path: &Path) -> Option<Vec<(PathBuf, Vec<u8>)>> {
    let dir = path.parent().unwrap_or(Path::new("."));
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "obj" => {
            let mtl = path.with_extension("mtl");
            let name = mtl.file_name()?.to_str()?;
            let (obj, text, textures) = super::write_obj_with_textures(model, name);
            let mut files = vec![
                (path.to_path_buf(), obj.into_bytes()),
                (mtl, text.into_bytes()),
            ];
            files.extend(
                textures
                    .into_iter()
                    .map(|(name, data)| (dir.join(name), data.to_vec())),
            );
            Some(files)
        }
        "dae" => {
            let (text, textures) = super::write_dae(model);
            let mut files = vec![(path.to_path_buf(), text.into_bytes())];
            files.extend(
                textures
                    .into_iter()
                    .map(|(name, data)| (dir.join(name), data.to_vec())),
            );
            Some(files)
        }
        _ => None,
    }
}

/// Refuse sidecar replacement by default. Callers approving a subset should use
/// `write_bundle_approved` to bind permission to the paths shown to the user.
pub fn write_bundle(files: &[(PathBuf, Vec<u8>)], allow_sidecars: bool) -> Result<(), IoError> {
    let approved = files.iter().enumerate().filter(|(i, _)| *i == 0 || allow_sidecars)
        .map(|(_, (p, _))| p.clone()).collect();
    write_bundle_approved(files, &approved)
}

/// Preflight every destination, then replace only explicitly approved paths.
/// New destinations use create_new, so late collisions cannot be overwritten.
pub fn write_bundle_approved(files: &[(PathBuf, Vec<u8>)], approved: &std::collections::HashSet<PathBuf>) -> Result<(), IoError> {
    let mut seen = std::collections::HashSet::new();
    for (path, _) in files {
        if !seen.insert(path) { return Err(IoError::Format("Conflicting export paths".into())); }
        if let Ok(meta) = std::fs::symlink_metadata(path)
            && (!meta.is_file() || !approved.contains(path)) {
            return Err(IoError::Format(format!("Export would replace an unapproved or non-regular file: {}", path.display())));
        }
    }
    for (path, bytes) in files {
        if approved.contains(path) { atomic_write(path, bytes)?; } else {
            use std::io::Write;
            std::fs::OpenOptions::new().write(true).create_new(true).open(path)?.write_all(bytes)?;
        }
    }
    Ok(())
}
