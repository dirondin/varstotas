//! Animation export (File → Export → Animation…): the frames of a scene tour or of an orbit
//! around the view's target, rendered one per frame (the view's style, or the path tracer with
//! a sample budget per frame), written as PNG files and encoded to video by ffmpeg.

use crate::nav::CameraAnimation;
use varstotas_kernel::glam::{DQuat, DVec3};
use varstotas_kernel::{Scene, SceneSettings};
use varstotas_render::Camera;
use std::path::{Path, PathBuf};

/// One frame to render.
#[derive(Clone, Debug)]
pub struct FrameSpec {
    pub camera: Camera,
    /// Apply this scene (style, tags, environment, shadows…) before the frame: the first frame
    /// of a scene's hold and of the transition into it.
    pub scene: Option<usize>,
    /// Values of components with parameters: (scene, next scene, 0..1 of the transition); a
    /// hold is (scene, scene, 1).
    pub rigs: Option<(usize, usize, f64)>,
}

/// The frames of the scenes in the animation at `fps`: a hold of `delay_secs` on each, a
/// `transition_secs` move to the next (a cut when transitions are off). The last scene is held
/// too. Empty when no scene is in the animation.
pub fn scene_frames(scenes: &[Scene], set: &SceneSettings, fps: f64) -> Vec<FrameSpec> {
    let order: Vec<usize> = (0..scenes.len()).filter(|&i| scenes[i].in_animation).collect();
    if order.is_empty() {
        return Vec::new();
    }
    let hold = set.delay_secs.max(0.0);
    let trans = if set.transitions { set.transition_secs.max(0.0) } else { 0.0 };
    // Segments: (start, length, scene, next scene for a transition).
    let mut segs: Vec<(f64, f64, usize, Option<usize>)> = Vec::new();
    let mut t = 0.0;
    for (k, &i) in order.iter().enumerate() {
        // At least one frame per scene.
        let h = hold.max(1.0 / fps);
        segs.push((t, h, i, None));
        t += h;
        if let Some(&j) = order.get(k + 1)
            && trans > 0.0
        {
            segs.push((t, trans, i, Some(j)));
            t += trans;
        }
    }
    let n = ((t * fps).round() as usize).max(1);
    let mut out = Vec::with_capacity(n);
    let mut applied: Option<usize> = None;
    for f in 0..n {
        let time = f as f64 / fps;
        let seg = segs.iter().rev().find(|s| s.0 <= time + 1e-9).unwrap_or(&segs[0]);
        let (start, len, i, next) = *seg;
        let (camera, scene, rigs) = match next {
            None => (crate::layout::scene_camera(&scenes[i]), i, (i, i, 1.0)),
            Some(j) => {
                let mut cam = crate::layout::scene_camera(&scenes[i]);
                let mut a = CameraAnimation::new(cam.clone(), crate::layout::scene_camera(&scenes[j]), len as f32);
                a.step((time - start) as f32, &mut cam);
                (cam, j, (i, j, ((time - start) / len).clamp(0.0, 1.0)))
            }
        };
        let change = applied != Some(scene);
        applied = Some(scene);
        out.push(FrameSpec { camera, scene: change.then_some(scene), rigs: Some(rigs) });
    }
    out
}

/// An orbit of `turns` around `base`'s target (about the vertical) over `seconds`.
pub fn orbit_frames(base: &Camera, seconds: f64, turns: f64, fps: f64) -> Vec<FrameSpec> {
    let n = ((seconds.max(0.1) * fps).round() as usize).max(1);
    (0..n)
        .map(|f| {
            let a = std::f64::consts::TAU * turns * f as f64 / n as f64;
            let q = DQuat::from_axis_angle(DVec3::Z, a);
            let mut c = base.clone();
            c.eye = base.target + q * (base.eye - base.target);
            c.up = q * base.up;
            FrameSpec { camera: c, scene: None, rigs: None }
        })
        .collect()
}

/// File of frame `i` in `dir` (the pattern ffmpeg reads).
pub fn frame_path(dir: &Path, i: usize) -> PathBuf {
    dir.join(format!("frame_{:05}.png", i + 1))
}

/// The ffmpeg executable, if installed (PATH, then the usual Homebrew / system places).
pub fn ffmpeg() -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"].map(PathBuf::from));
    let name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    dirs.into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

/// Encode the frames in `dir` into `out` (H.264, even size, yuv420p).
pub fn encode(ffmpeg: &Path, dir: &Path, fps: f64, out: &Path) -> Result<(), String> {
    let r = std::process::Command::new(ffmpeg)
        .args(["-y", "-loglevel", "error", "-framerate"])
        .arg(format!("{fps}"))
        .arg("-i")
        .arg(dir.join("frame_%05d.png"))
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2"])
        .arg(out)
        .output()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    if r.status.success() {
        Ok(())
    } else {
        Err(format!("ffmpeg: {}", String::from_utf8_lossy(&r.stderr).trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(name: &str, x: f64) -> Scene {
        Scene::camera(name, DVec3::new(x, -5000.0, 3000.0), DVec3::new(x, 0.0, 0.0), DVec3::Z, 35.0, false)
    }

    #[test]
    fn scene_tour_holds_and_moves() {
        let mut scenes = vec![scene("A", 0.0), scene("B", 1000.0), scene("C", 2000.0)];
        scenes[1].in_animation = false;
        let set = SceneSettings { transitions: true, transition_secs: 2.0, delay_secs: 1.0 };
        let f = scene_frames(&scenes, &set, 10.0);
        // A: hold 1 s, move 2 s, C: hold 1 s.
        assert_eq!(f.len(), 40);
        assert_eq!(f[0].scene, Some(0));
        assert_eq!(f[10].scene, Some(2), "the transition applies the next scene");
        assert!(f[1..10].iter().all(|x| x.scene.is_none()) && f[11..].iter().all(|x| x.scene.is_none()));
        assert_eq!(f[5].camera.target.x, 0.0);
        let mid = f[20].camera.target.x;
        assert!(mid > 500.0 && mid < 1500.0, "{mid}");
        assert_eq!(f[39].camera.target.x, 2000.0);
        // Cuts without transitions.
        let cut = scene_frames(&scenes, &SceneSettings { transitions: false, ..set }, 10.0);
        assert_eq!(cut.len(), 20);
        assert_eq!(cut[10].scene, Some(2));
        assert!(scene_frames(&[], &set, 10.0).is_empty());
    }

    #[test]
    fn orbit_turns_around_the_target() {
        let base = Camera { eye: DVec3::new(0.0, -1000.0, 500.0), target: DVec3::ZERO, ..Camera::default() };
        let f = orbit_frames(&base, 2.0, 1.0, 12.0);
        assert_eq!(f.len(), 24);
        let quarter = &f[6].camera;
        assert!((quarter.eye - DVec3::new(1000.0, 0.0, 500.0)).length() < 1e-6, "{}", quarter.eye);
        assert!(f.iter().all(|x| ((x.camera.eye - x.camera.target).length() - base.distance()).abs() < 1e-6));
        assert_eq!(frame_path(Path::new("/tmp/x"), 0), PathBuf::from("/tmp/x/frame_00001.png"));
    }
}
