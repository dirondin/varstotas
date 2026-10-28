//! The graphics API the window opens with (Vulkan, DirectX 12, Metal, OpenGL). A driver that
//! cannot work does not always say so: in Windows Sandbox Vulkan ends the process without a word
//! and DirectX 12 takes the whole sandbox down. So "Automatic" tries each API in a child process
//! (`varstotas --probe-gpu <api>`: adapter, device, a shader drawn and read back, with a time
//! limit) and keeps the first that works, a real GPU before a software one. The result is kept
//! in `gpu.json` in the data folder until the next version; an API whose start never drew a
//! frame (the process died on the way) is skipped from then on. Preferences → General picks an
//! API by hand and restarts the app with the document as it is (`restart.vrst`).
//! `WGPU_BACKEND` overrides all of it; scripted runs and macOS (only Metal) skip it.
use egui_wgpu::wgpu;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Api {
    #[default]
    Auto,
    Vulkan,
    Dx12,
    Metal,
    Gl,
}

impl Api {
    /// The APIs of this platform, in the order Automatic tries them.
    pub(crate) fn available() -> &'static [Api] {
        if cfg!(windows) {
            &[Api::Vulkan, Api::Dx12, Api::Gl]
        } else if cfg!(target_os = "macos") {
            &[Api::Metal]
        } else {
            &[Api::Vulkan, Api::Gl]
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Api::Auto => "",
            Api::Vulkan => "Vulkan",
            Api::Dx12 => "DirectX 12",
            Api::Metal => "Metal",
            Api::Gl => "OpenGL",
        }
    }

    fn arg(self) -> &'static str {
        match self {
            Api::Auto => "auto",
            Api::Vulkan => "vulkan",
            Api::Dx12 => "dx12",
            Api::Metal => "metal",
            Api::Gl => "gl",
        }
    }

    fn from_arg(s: &str) -> Option<Api> {
        [Api::Vulkan, Api::Dx12, Api::Metal, Api::Gl].into_iter().find(|a| a.arg() == s)
    }

    fn backends(self) -> wgpu::Backends {
        match self {
            Api::Auto => wgpu::Backends::PRIMARY | wgpu::Backends::GL,
            Api::Vulkan => wgpu::Backends::VULKAN,
            Api::Dx12 => wgpu::Backends::DX12,
            Api::Metal => wgpu::Backends::METAL,
            Api::Gl => wgpu::Backends::GL,
        }
    }

    fn of(backend: wgpu::Backend) -> Option<Api> {
        match backend {
            wgpu::Backend::Vulkan => Some(Api::Vulkan),
            wgpu::Backend::Dx12 => Some(Api::Dx12),
            wgpu::Backend::Metal => Some(Api::Metal),
            wgpu::Backend::Gl => Some(Api::Gl),
            _ => None,
        }
    }
}

/// `gpu.json` in the data folder.
#[derive(Default, Serialize, Deserialize)]
struct State {
    /// The user's choice.
    #[serde(default)]
    api: Api,
    /// The app version the rest was found with.
    #[serde(default)]
    version: String,
    /// What Automatic found.
    #[serde(default)]
    auto: Option<Api>,
    /// APIs whose start never drew a frame.
    #[serde(default)]
    failed: Vec<Api>,
    /// The API of a start that has not drawn its first frame yet.
    #[serde(default)]
    starting: Option<Api>,
}

fn state_path() -> Option<PathBuf> {
    crate::paths::data_dir().map(|d| d.join("gpu.json"))
}

fn load() -> State {
    let mut s: State = state_path()
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if s.version != env!("CARGO_PKG_VERSION") {
        // a new version may work where the old one did not
        s = State { api: s.api, version: env!("CARGO_PKG_VERSION").into(), ..State::default() };
    }
    s
}

fn store(s: &State) {
    if let Some(p) = state_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_vec_pretty(s) {
            let _ = varstotas_kernel::io::atomic_write(&p, text);
        }
    }
}

/// How this start chose: for Preferences.
#[derive(Default)]
struct Started {
    /// The choice was managed here (not WGPU_BACKEND, a script or macOS).
    managed: bool,
    /// The user's choice when the app started.
    choice: Api,
    /// The user's choice now (for the next start).
    saved: Api,
    /// "Vulkan · NVIDIA GeForce …", once the first frame is drawn.
    in_use: Option<String>,
    /// The chosen API failed last time, so Automatic took over.
    fell_back: Option<Api>,
}

fn started() -> &'static Mutex<Started> {
    static S: OnceLock<Mutex<Started>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Started::default()))
}

fn managed() -> bool {
    !cfg!(target_os = "macos") && std::env::var_os("WGPU_BACKEND").is_none() && std::env::var_os("VARSTOTAS_SCRIPT").is_none()
}

/// The backends the window may use this start (None: wgpu's own choice).
pub(crate) fn choose() -> Option<wgpu::Backends> {
    if !managed() {
        return None;
    }
    let mut s = load();
    if let Some(dead) = s.starting.take() {
        log::warn!("the last start with {} did not draw a frame; skipping it", dead.name());
        if !s.failed.contains(&dead) {
            s.failed.push(dead);
        }
        if s.auto == Some(dead) {
            s.auto = None;
        }
    }
    let mut fell_back = None;
    let api = if s.api != Api::Auto && !s.failed.contains(&s.api) {
        Some(s.api)
    } else {
        if s.api != Api::Auto {
            fell_back = Some(s.api);
        }
        if s.auto.is_none() {
            s.auto = probe_all(&s.failed);
        }
        s.auto
    };
    s.starting = api;
    store(&s);
    *started().lock().unwrap() = Started { managed: true, choice: s.api, saved: s.api, fell_back, ..Started::default() };
    let api = api?;
    log::info!("graphics: {}", api.name());
    Some(api.backends())
}

/// Try the platform's APIs in turn; the first that works on a GPU, else the first that works.
fn probe_all(failed: &[Api]) -> Option<Api> {
    let mut software = None;
    // Windows Sandbox: its virtual GPU takes the whole sandbox down under DirectX 12
    let sandbox = cfg!(windows) && std::env::var("USERNAME").is_ok_and(|u| u.eq_ignore_ascii_case("WDAGUtilityAccount"));
    for &api in Api::available().iter().filter(|a| !failed.contains(a) && !(sandbox && **a == Api::Dx12)) {
        match run_probe(api) {
            Ok(kind) if kind == "Cpu" => {
                log::info!("graphics probe {}: software only", api.name());
                software = software.or(Some(api));
            }
            Ok(kind) => {
                log::info!("graphics probe {}: {kind}", api.name());
                return Some(api);
            }
            Err(e) => log::warn!("graphics probe {}: {e}", api.name()),
        }
    }
    software
}

/// `varstotas --probe-gpu <api>` in a child process: the adapter's device type, or why not.
fn run_probe(api: Api) -> Result<String, String> {
    const LIMIT: Duration = Duration::from_secs(20);
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["--probe-gpu", api.arg()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => break status,
            None if start.elapsed() > LIMIT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("no answer in 20 s".into());
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    let mut out = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut out);
    }
    let line = out.lines().last().unwrap_or_default();
    match line.split_once(' ') {
        Some(("ok", kind)) if status.success() => Ok(kind.to_string()),
        Some(("error", e)) => Err(e.to_string()),
        _ => Err(format!("ended with {status}")),
    }
}

/// The child side of [`run_probe`]: `ok <device type>` or `error <why>` on stdout.
pub(crate) fn probe_main(arg: &str) -> ! {
    let code = match Api::from_arg(arg).ok_or_else(|| format!("unknown API {arg}")).and_then(probe) {
        Ok(kind) => {
            println!("ok {kind}");
            0
        }
        Err(e) => {
            println!("error {}", e.replace('\n', " "));
            1
        }
    };
    std::process::exit(code)
}

/// wgpu's native futures are ready at once.
fn ready<F: std::future::Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::yield_now();
    }
}

/// Open `api` on the best adapter, draw a triangle and read it back.
fn probe(api: Api) -> Result<String, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: api.backends(),
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        backend_options: wgpu::BackendOptions::from_env_or_default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        display: None,
    });
    let adapter = ready(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .map_err(|e| e.to_string())?;
    let info = adapter.get_info();
    // what the window asks for (see main)
    let required_limits = if info.backend == wgpu::Backend::Gl {
        adapter.limits()
    } else {
        wgpu::Limits { max_texture_dimension_2d: 8192, ..wgpu::Limits::default() }
    };
    let (device, queue) = ready(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("varstotas probe"),
        required_limits,
        ..Default::default()
    }))
    .map_err(|e| e.to_string())?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("varstotas probe"),
        source: wgpu::ShaderSource::Wgsl(
            "@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
                return vec4f(f32(i & 1u) * 4.0 - 1.0, f32(i >> 1u) * 4.0 - 1.0, 0.0, 1.0);
            }
            @fragment fn fs() -> @location(0) vec4f { return vec4f(0.0, 1.0, 0.0, 1.0); }"
                .into(),
        ),
    });
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("varstotas probe"),
        layout: None,
        vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        multiview_mask: None,
        cache: None,
    });
    const SIZE: u32 = 64;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("varstotas probe"),
        size: wgpu::Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("varstotas probe"),
        size: u64::from(SIZE * SIZE * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas probe") });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("varstotas probe"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(SIZE * 4), rows_per_image: Some(SIZE) },
        },
        wgpu::Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 },
    );
    let submission = queue.submit(Some(enc.finish()));
    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: None }).map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    let data = slice.get_mapped_range().map_err(|e| e.to_string())?;
    let center = (((SIZE / 2) * SIZE + SIZE / 2) * 4) as usize;
    if data[center..center + 4] != [0, 255, 0, 255] {
        return Err(format!("drew {:?} instead of green", &data[center..center + 4]));
    }
    Ok(format!("{:?}", info.device_type))
}

/// Called on every frame; the first one tells that the API works.
pub(crate) fn frame_drawn(frame: &eframe::Frame) {
    static DONE: OnceLock<()> = OnceLock::new();
    if DONE.set(()).is_err() {
        return;
    }
    let info = frame.wgpu_render_state().map(|rs| rs.adapter.get_info());
    let mut st = started().lock().unwrap();
    if let Some(info) = &info {
        st.in_use = Some(format!("{} · {}", Api::of(info.backend).map_or("?", Api::name), info.name));
        log::info!("graphics in use: {:?} {} ({} {})", info.backend, info.name, info.driver, info.driver_info);
    }
    if st.managed {
        let mut s = load();
        s.starting = None;
        store(&s);
    }
}

/// For Preferences: whether the API can be chosen here, the saved choice and the one this start
/// had, what is in use, and the API that failed last time (Automatic took over).
pub(crate) struct Status {
    pub(crate) managed: bool,
    pub(crate) choice: Api,
    pub(crate) running_choice: Api,
    pub(crate) in_use: Option<String>,
    pub(crate) fell_back: Option<Api>,
}

pub(crate) fn status() -> Status {
    let st = started().lock().unwrap();
    Status {
        managed: st.managed,
        choice: st.saved,
        running_choice: st.choice,
        in_use: st.in_use.clone(),
        fell_back: st.fell_back,
    }
}

/// Save the choice for the next start; picking an API by hand gives it another chance.
pub(crate) fn set_choice(api: Api) {
    let mut s = load();
    s.api = api;
    s.failed.retain(|&a| a != api);
    store(&s);
    started().lock().unwrap().saved = api;
}

/// The document handed over by a restart (Preferences → Graphics API).
#[derive(Serialize, Deserialize)]
pub(crate) struct Restart {
    /// The document's file.
    pub(crate) path: Option<PathBuf>,
    /// Its unsaved state is in `restart.vrst`.
    pub(crate) unsaved: bool,
    /// eye, target, up, vertical field of view, parallel (0 / 1), parallel height
    pub(crate) camera: Option<[f64; 12]>,
}

fn restart_paths() -> Option<(PathBuf, PathBuf)> {
    let d = crate::paths::data_dir()?;
    Some((d.join("restart.json"), d.join("restart.vrst")))
}

/// Start a new instance of the app that takes over `restart` (and `model`, when unsaved); the
/// caller closes this one. The new one waits until this process is gone (its stdin closes).
pub(crate) fn restart(restart: &Restart, model: Option<&varstotas_kernel::Model>) -> Result<(), String> {
    let (json, vrst) = restart_paths().ok_or("no data folder")?;
    if let Some(dir) = json.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if let Some(model) = model {
        crate::autosave::write_model(model, &vrst)?;
    }
    varstotas_kernel::io::atomic_write(&json, serde_json::to_vec(restart).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    // an AppImage is unmounted when it exits: start the AppImage file itself
    let exe = match crate::appimage::running() {
        Some(app) => app,
        None => std::env::current_exe().map_err(|e| e.to_string())?,
    };
    let child = std::process::Command::new(exe)
        .env("VARSTOTAS_RESTART", "1")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    // the pipe closes when this process exits
    std::mem::forget(child.stdin);
    Ok(())
}

/// In a restarted instance (before the window): wait until the old one is gone, so its
/// preferences are written and its window closed.
pub(crate) fn wait_for_restart() {
    if std::env::var_os("VARSTOTAS_RESTART").is_none() {
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = std::io::stdin().read_to_end(&mut Vec::new());
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(Duration::from_secs(15));
}

/// The document a restart handed over (once): its file, and the unsaved model if there is one.
pub(crate) fn take_restart() -> Option<(Restart, Option<varstotas_kernel::Model>)> {
    std::env::var_os("VARSTOTAS_RESTART")?;
    let (json, vrst) = restart_paths()?;
    let r: Restart = std::fs::read(&json).ok().and_then(|b| serde_json::from_slice(&b).ok())?;
    let _ = std::fs::remove_file(&json);
    let model = r.unsaved.then(|| varstotas_kernel::io::load(&vrst).ok()).flatten();
    let _ = std::fs::remove_file(&vrst);
    Some((r, model))
}

impl crate::app::VarstotasApp {
    /// Restart with the graphics API chosen in Preferences, keeping the document as it is.
    pub(crate) fn restart_for_graphics(&mut self, ctx: &egui::Context) {
        let c = &self.camera;
        let camera = [
            c.eye.x, c.eye.y, c.eye.z, c.target.x, c.target.y, c.target.z, c.up.x, c.up.y, c.up.z,
            c.fov_y_deg, if c.ortho { 1.0 } else { 0.0 }, c.ortho_height,
        ];
        let r = Restart { path: self.doc.path.clone(), unsaved: self.doc.dirty, camera: Some(camera) };
        match restart(&r, self.doc.dirty.then_some(&self.doc.model)) {
            Ok(()) => {
                self.restarting = true;
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => self.flash_error(varstotas_i18n::tr!("prefs-gpu-restart-failed", error = e)),
        }
    }

    /// Take over the document of the instance that restarted.
    pub(crate) fn take_over(&mut self, r: Restart, model: Option<varstotas_kernel::Model>) {
        match (model, &r.path) {
            (Some(model), _) => {
                self.replace_document(crate::document::Document::from_model(model, r.path.clone()));
                self.doc.dirty = true;
            }
            (None, Some(path)) => self.open_path(path),
            (None, None) => {}
        }
        self.show_start = false;
        if let Some(c) = r.camera {
            let v = |i: usize| glam::DVec3::new(c[i], c[i + 1], c[i + 2]);
            self.camera.eye = v(0);
            self.camera.target = v(3);
            self.camera.up = v(6);
            self.camera.fov_y_deg = c[9];
            self.camera.ortho = c[10] != 0.0;
            self.camera.ortho_height = c[11];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_names_round_trip() {
        for &a in &[Api::Vulkan, Api::Dx12, Api::Metal, Api::Gl] {
            assert_eq!(Api::from_arg(a.arg()), Some(a));
        }
        assert_eq!(Api::from_arg("auto"), None);
        let s: State = serde_json::from_str(r#"{"api":"dx12","failed":["vulkan"]}"#).unwrap();
        assert_eq!((s.api, s.failed), (Api::Dx12, vec![Api::Vulkan]));
    }
}
