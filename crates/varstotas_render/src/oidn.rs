//! Intel Open Image Denoise, loaded at run time when installed (Homebrew
//! `open-image-denoise`, the library next to the executable, Preferences → Render, or
//! `VARSTOTAS_OIDN` = its path).
//! The path tracer's final denoise uses it instead of the built-in à-trous filter.

use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

type Device = *mut c_void;
type Filter = *mut c_void;

const DEVICE_TYPE_DEFAULT: i32 = 0;
const FORMAT_FLOAT3: i32 = 3;
const QUALITY_HIGH: i32 = 6;

struct Api {
    _lib: libloading::Library,
    device: Device,
    new_filter: unsafe extern "C" fn(Device, *const c_char) -> Filter,
    set_image: unsafe extern "C" fn(Filter, *const c_char, *mut c_void, i32, usize, usize, usize, usize, usize),
    set_bool: unsafe extern "C" fn(Filter, *const c_char, bool),
    set_int: unsafe extern "C" fn(Filter, *const c_char, i32),
    commit: unsafe extern "C" fn(Filter),
    execute: unsafe extern "C" fn(Filter),
    release: unsafe extern "C" fn(Filter),
    error: unsafe extern "C" fn(Device, *mut *const c_char) -> i32,
}

// SAFETY: the device is only used under the mutex below, one call at a time.
unsafe impl Send for Api {}

fn candidates() -> Vec<std::path::PathBuf> {
    let mut v: Vec<PathBuf> = CUSTOM.lock().ok().and_then(|c| c.clone()).into_iter().collect();
    if let Some(p) = std::env::var_os("VARSTOTAS_OIDN") {
        v.push(p.into());
    }
    let names: &[&str] = if cfg!(target_os = "macos") {
        &["libOpenImageDenoise.2.dylib", "libOpenImageDenoise.dylib"]
    } else if cfg!(windows) {
        &["OpenImageDenoise.dll"]
    } else {
        &["libOpenImageDenoise.so.2", "libOpenImageDenoise.so"]
    };
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.to_path_buf())) {
        v.extend(names.iter().map(|n| dir.join(n)));
    }
    for dir in ["/opt/homebrew/lib", "/usr/local/lib"] {
        v.extend(names.iter().map(|n| std::path::Path::new(dir).join(n)));
    }
    // The system's search path last.
    v.extend(names.iter().map(std::path::PathBuf::from));
    v
}

fn load() -> Option<Api> {
    if std::env::var_os("VARSTOTAS_NO_OIDN").is_some() {
        return None;
    }
    for path in candidates() {
        // SAFETY: loading OIDN runs its initialisers only; the symbols are used with the
        // signatures of oidn.h (2.x).
        let Ok(lib) = (unsafe { libloading::Library::new(&path) }) else { continue };
        let api = unsafe {
            let new_device: libloading::Symbol<unsafe extern "C" fn(i32) -> Device> = lib.get(b"oidnNewDevice\0").ok()?;
            let commit_device: libloading::Symbol<unsafe extern "C" fn(Device)> = lib.get(b"oidnCommitDevice\0").ok()?;
            let device = new_device(DEVICE_TYPE_DEFAULT);
            if device.is_null() {
                continue;
            }
            commit_device(device);
            Api {
                device,
                new_filter: *lib.get(b"oidnNewFilter\0").ok()?,
                set_image: *lib.get(b"oidnSetSharedFilterImage\0").ok()?,
                set_bool: *lib.get(b"oidnSetFilterBool\0").ok()?,
                set_int: *lib.get(b"oidnSetFilterInt\0").ok()?,
                commit: *lib.get(b"oidnCommitFilter\0").ok()?,
                execute: *lib.get(b"oidnExecuteFilter\0").ok()?,
                release: *lib.get(b"oidnReleaseFilter\0").ok()?,
                error: *lib.get(b"oidnGetDeviceError\0").ok()?,
                _lib: lib,
            }
        };
        if let Some(e) = api.take_error() {
            log::warn!("denoiser: {}: {e}", path.display());
            continue;
        }
        log::info!("denoiser: Open Image Denoise ({})", path.display());
        if let Ok(mut l) = LOADED.lock() {
            *l = Some(path);
        }
        return Some(api);
    }
    if let Ok(mut l) = LOADED.lock() {
        *l = None;
    }
    None
}

impl Api {
    fn take_error(&self) -> Option<String> {
        let mut msg: *const c_char = std::ptr::null();
        // SAFETY: a valid device; the message is owned by OIDN until the next call.
        let code = unsafe { (self.error)(self.device, &mut msg) };
        (code != 0).then(|| if msg.is_null() { format!("error {code}") } else { unsafe { CStr::from_ptr(msg) }.to_string_lossy().into_owned() })
    }
}

/// The library chosen in Preferences (tried first).
static CUSTOM: Mutex<Option<PathBuf>> = Mutex::new(None);
static API: OnceLock<Mutex<Option<Api>>> = OnceLock::new();
/// The path of the loaded library (read without waiting for a denoise in progress).
static LOADED: Mutex<Option<PathBuf>> = Mutex::new(None);

fn api() -> &'static Mutex<Option<Api>> {
    API.get_or_init(|| Mutex::new(load()))
}

/// Use the library at `p` first (None = only the usual places); loads it again when changed.
pub fn set_library(p: Option<&Path>) {
    let Ok(mut c) = CUSTOM.lock() else { return };
    if c.as_deref() == p {
        return;
    }
    *c = p.map(Path::to_path_buf);
    drop(c);
    if let Some(m) = API.get()
        && let Ok(mut a) = m.lock()
    {
        // the old device is never released: unloading its library under it is not safe
        std::mem::forget(a.take());
        *a = load();
    }
}

/// The library in use.
pub fn library() -> Option<PathBuf> {
    api();
    LOADED.lock().ok().and_then(|l| l.clone())
}

/// Open Image Denoise is installed.
pub fn available() -> bool {
    api().lock().map(|a| a.is_some()).unwrap_or(false)
}

/// Denoise a linear HDR image (RGB per pixel) with its first-hit albedo and normal (same
/// layout); `None` when OIDN is missing or fails.
pub(crate) fn denoise(w: usize, h: usize, color: &mut [f32], albedo: &mut [f32], normal: &mut [f32]) -> Option<Vec<f32>> {
    let n = w * h * 3;
    if color.len() != n || albedo.len() != n || normal.len() != n {
        return None;
    }
    let guard = api().lock().ok()?;
    let api = guard.as_ref()?;
    let mut out = vec![0f32; n];
    // SAFETY: the buffers hold w × h float3 pixels and outlive the filter's execution.
    unsafe {
        let f = (api.new_filter)(api.device, c"RT".as_ptr());
        if f.is_null() {
            return None;
        }
        let img = |name: &CStr, p: *mut f32| (api.set_image)(f, name.as_ptr(), p.cast(), FORMAT_FLOAT3, w, h, 0, 0, 0);
        img(c"color", color.as_mut_ptr());
        img(c"albedo", albedo.as_mut_ptr());
        img(c"normal", normal.as_mut_ptr());
        img(c"output", out.as_mut_ptr());
        (api.set_bool)(f, c"hdr".as_ptr(), true);
        (api.set_int)(f, c"quality".as_ptr(), QUALITY_HIGH);
        (api.commit)(f);
        (api.execute)(f);
        (api.release)(f);
    }
    if let Some(e) = api.take_error() {
        log::warn!("denoiser: {e}");
        return None;
    }
    Some(out)
}
