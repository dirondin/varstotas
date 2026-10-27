use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use varstotas_kernel::{Model, io};
fn dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "varstotas-backup-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
#[test]
fn explicit_save_rotates_one_exact_reopenable_previous_version() {
    let dir = dir();
    let path = dir.join("Стол с размерами.vrst");
    let backup = dir.join("model.bak.vrst");
    let mut model = Model::new();
    io::save_with_backup(&model, &path, &backup).unwrap();
    assert!(!backup.exists());
    let first = std::fs::read(&path).unwrap();
    model.add_tag("Second version");
    io::save_with_backup(&model, &path, &backup).unwrap();
    assert_eq!(std::fs::read(&backup).unwrap(), first);
    assert!(io::load(&backup).unwrap().tags.is_empty());
    io::save_with_backup(&model, &path, &backup).unwrap();
    assert_eq!(
        std::fs::read(&backup).unwrap(),
        first,
        "an unchanged save keeps the last distinct version"
    );
    let second = std::fs::read(&path).unwrap();
    model.add_tag("Third version");
    io::save_with_backup(&model, &path, &backup).unwrap();
    assert_eq!(std::fs::read(&backup).unwrap(), second);
    assert_eq!(io::load(&backup).unwrap().tags.len(), 1);
    assert_eq!(io::load(&path).unwrap().tags.len(), 2);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn failed_backup_does_not_replace_the_original_document() {
    let dir = dir();
    let path = dir.join("model.vrst");
    let backup = dir.join("model.bak.vrst");
    let mut model = Model::new();
    io::save(&model, &path).unwrap();
    let original = std::fs::read(&path).unwrap();
    std::fs::create_dir_all(&backup).unwrap();
    model.add_tag("New");
    assert!(io::save_with_backup(&model, &path, &backup).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    std::fs::remove_dir_all(dir).unwrap();
}
#[cfg(unix)]
#[test]
fn backup_and_document_symlinks_are_not_followed() {
    let dir = dir();
    let path = dir.join("model.vrst");
    let backup = dir.join("model.bak.vrst");
    let outside = dir.join("unrelated.vrst");
    io::save(&Model::new(), &path).unwrap();
    std::fs::write(&outside, b"keep").unwrap();
    let original = std::fs::read(&path).unwrap();
    std::os::unix::fs::symlink(&outside, &backup).unwrap();
    let mut changed = Model::new();
    changed.add_tag("Changed");
    assert!(io::save_with_backup(&changed, &path, &backup).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read(&outside).unwrap(), b"keep");
    let alias = dir.join("alias.vrst");
    std::os::unix::fs::symlink(&path, &alias).unwrap();
    assert!(io::save_with_backup(&Model::new(), &alias, &backup).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    std::fs::remove_dir_all(dir).unwrap();
}
