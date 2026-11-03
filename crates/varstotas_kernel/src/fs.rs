//! The file system the app works with. The desktop builds use the real one (`std::fs`, re-exported
//! here as is); the web build (wasm32) has no file system, so there it is kept in memory: the
//! files the user opens are put in, what is written is handed to the page (`set_on_write`: a
//! download, or the browser's storage for the data folder). The calls are the ones of `std::fs`
//! the app makes, so the same code runs in both.

#[cfg(not(target_arch = "wasm32"))]
pub use std::fs::*;

#[cfg(target_arch = "wasm32")]
pub use web::*;

use std::path::Path;

/// `Path::exists` through this file system (the std one never finds a file in the web build).
pub fn exists(path: &Path) -> bool {
    metadata(path).is_ok()
}

/// `Path::is_file` through this file system.
pub fn is_file(path: &Path) -> bool {
    metadata(path).is_ok_and(|m| m.is_file())
}

/// `Path::is_dir` through this file system.
pub fn is_dir(path: &Path) -> bool {
    metadata(path).is_ok_and(|m| m.is_dir())
}

/// The web build's file system: files and folders in memory under `/`.
#[cfg(target_arch = "wasm32")]
mod web {
    use std::collections::{BTreeMap, BTreeSet};
    use std::ffi::OsString;
    use std::io::{self, ErrorKind};
    use std::path::{Component, Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::SystemTime;

    struct Node {
        data: Vec<u8>,
        modified: SystemTime,
    }

    #[derive(Default)]
    struct Fs {
        files: BTreeMap<PathBuf, Node>,
        dirs: BTreeSet<PathBuf>,
    }

    static FS: Mutex<Option<Fs>> = Mutex::new(None);
    static ON_WRITE: OnceLock<fn(&Path, &[u8])> = OnceLock::new();

    /// Called with every file written, copied or renamed to (its path and contents).
    pub fn set_on_write(f: fn(&Path, &[u8])) {
        let _ = ON_WRITE.set(f);
    }

    fn with<R>(f: impl FnOnce(&mut Fs) -> R) -> R {
        let mut g = FS.lock().unwrap_or_else(|e| e.into_inner());
        f(g.get_or_insert_with(Fs::default))
    }

    fn now() -> SystemTime {
        let since = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).unwrap_or_default();
        std::time::UNIX_EPOCH + since
    }

    /// An absolute path without `.` and `..` (relative ones are taken from `/`).
    fn norm(p: &Path) -> PathBuf {
        let mut out = PathBuf::from("/");
        for c in p.components() {
            match c {
                Component::Normal(s) => out.push(s),
                Component::ParentDir => {
                    out.pop();
                }
                _ => {}
            }
        }
        out
    }

    fn not_found(p: &Path) -> io::Error {
        io::Error::new(ErrorKind::NotFound, format!("{}: not found", p.display()))
    }

    fn written(p: &Path, data: &[u8]) {
        if let Some(f) = ON_WRITE.get() {
            f(p, data);
        }
    }

    impl Fs {
        fn is_dir(&self, p: &Path) -> bool {
            p == Path::new("/") || self.dirs.contains(p) || self.files.keys().any(|f| f.starts_with(p) && f != p)
        }
        fn add_parents(&mut self, p: &Path) {
            let mut d = p.parent();
            while let Some(x) = d.filter(|x| *x != Path::new("/")) {
                self.dirs.insert(x.to_path_buf());
                d = x.parent();
            }
        }
        fn put(&mut self, p: PathBuf, data: Vec<u8>) {
            self.add_parents(&p);
            self.files.insert(p, Node { data, modified: now() });
        }
    }

    /// Put a file in (one the user opened in the browser), without handing it back to the page.
    pub fn insert(path: impl AsRef<Path>, data: Vec<u8>) {
        let p = norm(path.as_ref());
        with(|fs| fs.put(p, data));
    }

    pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let p = norm(path.as_ref());
        with(|fs| fs.files.get(&p).map(|n| n.data.clone())).ok_or_else(|| not_found(&p))
    }

    pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
        String::from_utf8(read(path)?).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    }

    pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
        let p = norm(path.as_ref());
        let data = contents.as_ref().to_vec();
        with(|fs| fs.put(p.clone(), data.clone()));
        written(&p, &data);
        Ok(())
    }

    pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
        let p = norm(path.as_ref());
        with(|fs| {
            fs.add_parents(&p.join("x"));
        });
        Ok(())
    }

    pub fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
        let p = norm(path.as_ref());
        with(|fs| {
            if fs.is_dir(&p) || fs.files.contains_key(&p) {
                return Err(io::Error::new(ErrorKind::AlreadyExists, format!("{}: exists", p.display())));
            }
            fs.add_parents(&p.join("x"));
            Ok(())
        })
    }

    pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
        let p = norm(path.as_ref());
        with(|fs| fs.files.remove(&p).map(|_| ())).ok_or_else(|| not_found(&p))
    }

    pub fn remove_dir(path: impl AsRef<Path>) -> io::Result<()> {
        let p = norm(path.as_ref());
        with(|fs| {
            if fs.files.keys().any(|f| f.starts_with(&p)) || fs.dirs.iter().any(|d| d.starts_with(&p) && *d != p) {
                return Err(io::Error::other(format!("{}: not empty", p.display())));
            }
            fs.dirs.remove(&p).then_some(()).ok_or_else(|| not_found(&p))
        })
    }

    pub fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
        let p = norm(path.as_ref());
        with(|fs| {
            let had = fs.is_dir(&p);
            fs.files.retain(|f, _| !f.starts_with(&p));
            fs.dirs.retain(|d| !d.starts_with(&p));
            had.then_some(()).ok_or_else(|| not_found(&p))
        })
    }

    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let (a, b) = (norm(from.as_ref()), norm(to.as_ref()));
        let data = with(|fs| {
            if let Some(n) = fs.files.remove(&a) {
                let data = n.data.clone();
                fs.add_parents(&b);
                fs.files.insert(b.clone(), n);
                return Ok(Some(data));
            }
            if !fs.is_dir(&a) {
                return Err(not_found(&a));
            }
            // a folder: everything in it moves
            let files: Vec<PathBuf> = fs.files.keys().filter(|f| f.starts_with(&a)).cloned().collect();
            for f in files {
                let n = fs.files.remove(&f).expect("listed");
                let to = b.join(f.strip_prefix(&a).expect("inside"));
                fs.add_parents(&to);
                fs.files.insert(to, n);
            }
            let dirs: Vec<PathBuf> = fs.dirs.iter().filter(|d| d.starts_with(&a)).cloned().collect();
            for d in dirs {
                fs.dirs.remove(&d);
                fs.dirs.insert(b.join(d.strip_prefix(&a).expect("inside")));
            }
            fs.add_parents(&b.join("x"));
            Ok(None)
        })?;
        if let Some(data) = data {
            written(&b, &data);
        }
        Ok(())
    }

    pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
        let data = read(from)?;
        let n = data.len() as u64;
        write(to, data)?;
        Ok(n)
    }

    pub fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
        let p = norm(path.as_ref());
        super::exists(&p).then_some(p.clone()).ok_or_else(|| not_found(&p))
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FileType {
        dir: bool,
    }

    impl FileType {
        pub fn is_dir(&self) -> bool {
            self.dir
        }
        pub fn is_file(&self) -> bool {
            !self.dir
        }
        pub fn is_symlink(&self) -> bool {
            false
        }
    }

    #[derive(Clone, Debug)]
    #[allow(clippy::len_without_is_empty)] // std's has none either
    pub struct Metadata {
        dir: bool,
        len: u64,
        modified: SystemTime,
    }

    impl Metadata {
        pub fn is_dir(&self) -> bool {
            self.dir
        }
        pub fn is_file(&self) -> bool {
            !self.dir
        }
        pub fn is_symlink(&self) -> bool {
            false
        }
        pub fn len(&self) -> u64 {
            self.len
        }
        pub fn modified(&self) -> io::Result<SystemTime> {
            Ok(self.modified)
        }
        pub fn file_type(&self) -> FileType {
            FileType { dir: self.dir }
        }
    }

    pub fn metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
        let p = norm(path.as_ref());
        with(|fs| {
            if let Some(n) = fs.files.get(&p) {
                return Ok(Metadata { dir: false, len: n.data.len() as u64, modified: n.modified });
            }
            fs.is_dir(&p).then_some(Metadata { dir: true, len: 0, modified: std::time::UNIX_EPOCH }).ok_or_else(|| not_found(&p))
        })
    }

    pub fn symlink_metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
        metadata(path)
    }

    pub struct DirEntry {
        path: PathBuf,
    }

    impl DirEntry {
        pub fn path(&self) -> PathBuf {
            self.path.clone()
        }
        pub fn file_name(&self) -> OsString {
            self.path.file_name().map(OsString::from).unwrap_or_default()
        }
        pub fn metadata(&self) -> io::Result<Metadata> {
            metadata(&self.path)
        }
        pub fn file_type(&self) -> io::Result<FileType> {
            Ok(metadata(&self.path)?.file_type())
        }
    }

    pub type ReadDir = std::vec::IntoIter<io::Result<DirEntry>>;

    /// The files and folders right inside `path`, by name.
    pub fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
        let p = norm(path.as_ref());
        with(|fs| {
            if !fs.is_dir(&p) {
                return Err(not_found(&p));
            }
            let mut names = BTreeSet::new();
            for f in fs.files.keys().chain(fs.dirs.iter()) {
                if let Ok(rest) = f.strip_prefix(&p)
                    && let Some(Component::Normal(first)) = rest.components().next()
                {
                    names.insert(p.join(first));
                }
            }
            Ok(names.into_iter().map(|path| Ok(DirEntry { path })).collect::<Vec<_>>().into_iter())
        })
    }

    /// A file being written: its bytes go in when it is dropped (or flushed).
    pub struct File {
        path: PathBuf,
        data: Vec<u8>,
        dirty: bool,
    }

    impl File {
        /// An empty file now (not handed to the page until something is written into it).
        pub fn create(path: impl AsRef<Path>) -> io::Result<File> {
            let path = norm(path.as_ref());
            insert(&path, Vec::new());
            Ok(File { path, data: Vec::new(), dirty: false })
        }
        pub fn open(path: impl AsRef<Path>) -> io::Result<File> {
            let data = read(&path)?;
            Ok(File { path: norm(path.as_ref()), data, dirty: false })
        }
        pub fn options() -> OpenOptions {
            OpenOptions::new()
        }
        pub fn metadata(&self) -> io::Result<Metadata> {
            Ok(Metadata { dir: false, len: self.data.len() as u64, modified: now() })
        }
        pub fn sync_all(&mut self) -> io::Result<()> {
            std::io::Write::flush(self)
        }
    }

    impl std::io::Write for File {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.data.extend_from_slice(buf);
            self.dirty = true;
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            if std::mem::take(&mut self.dirty) {
                write(&self.path, &self.data)?;
            }
            Ok(())
        }
    }

    impl std::io::Read for File {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = buf.len().min(self.data.len());
            buf[..n].copy_from_slice(&self.data[..n]);
            self.data.drain(..n);
            Ok(n)
        }
    }

    impl Drop for File {
        fn drop(&mut self) {
            let _ = std::io::Write::flush(self);
        }
    }

    #[derive(Default)]
    pub struct OpenOptions {
        create_new: bool,
    }

    impl OpenOptions {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn write(&mut self, _write: bool) -> &mut Self {
            self
        }
        pub fn read(&mut self, _read: bool) -> &mut Self {
            self
        }
        pub fn create(&mut self, _create: bool) -> &mut Self {
            self
        }
        pub fn truncate(&mut self, _truncate: bool) -> &mut Self {
            self
        }
        pub fn create_new(&mut self, create_new: bool) -> &mut Self {
            self.create_new = create_new;
            self
        }
        pub fn open(&self, path: impl AsRef<Path>) -> io::Result<File> {
            if self.create_new && super::exists(path.as_ref()) {
                return Err(io::Error::new(ErrorKind::AlreadyExists, format!("{}: exists", path.as_ref().display())));
            }
            File::create(path)
        }
    }
}
