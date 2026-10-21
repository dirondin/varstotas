//! The application log (Window → Log): errors and warnings from every part of the app in one
//! place — status bar errors, scripts, generators, component parameters, plugins, import /
//! export, libraries, downloads. Everything the `log` crate gets at warning level from Varstotas's
//! own crates (and errors from any crate) is kept too; [`init`] installs that logger in front
//! of env_logger, which still prints to the terminal as before.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Entries kept (the oldest are dropped).
const KEEP: usize = 2000;

/// The same message again within this many seconds only counts up.
const REPEAT_SECS: u64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warn,
}

impl Level {
    pub fn label(self) -> String {
        match self {
            Level::Error => varstotas_i18n::tr!("log-level-error"),
            Level::Warn => varstotas_i18n::tr!("log-level-warning"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    /// Seconds since the Unix epoch (of the last repeat).
    pub at: u64,
    pub level: Level,
    /// Where it came from: "Script Console", "Import", "library" …
    pub source: String,
    pub text: String,
    /// How many times it came in a row.
    pub count: u32,
}

struct Store {
    entries: VecDeque<Entry>,
    /// Errors since the log was last looked at.
    unseen: usize,
    /// Bumped on every change (the window redraws its copy then).
    version: u64,
}

static LOG: Mutex<Store> = Mutex::new(Store { entries: VecDeque::new(), unseen: 0, version: 0 });

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Add an entry (a repeat of the last one only counts up).
pub fn push(level: Level, source: &str, text: &str) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    let at = now();
    let Ok(mut s) = LOG.lock() else { return };
    s.version += 1;
    if level == Level::Error {
        s.unseen += 1;
    }
    if let Some(last) = s.entries.back_mut()
        && last.level == level
        && last.source == source
        && last.text == text
        && at.saturating_sub(last.at) <= REPEAT_SECS
    {
        last.count += 1;
        last.at = at;
        return;
    }
    s.entries.push_back(Entry { at, level, source: source.to_string(), text: text.to_string(), count: 1 });
    while s.entries.len() > KEEP {
        s.entries.pop_front();
    }
}

pub fn error(source: &str, text: &str) {
    push(Level::Error, source, text);
}

/// The entries and the version they are of.
pub fn snapshot() -> (u64, Vec<Entry>) {
    LOG.lock().map(|s| (s.version, s.entries.iter().cloned().collect())).unwrap_or_default()
}

pub fn version() -> u64 {
    LOG.lock().map(|s| s.version).unwrap_or(0)
}

/// Errors added since [`mark_seen`].
pub fn unseen() -> usize {
    LOG.lock().map(|s| s.unseen).unwrap_or(0)
}

pub fn mark_seen() {
    if let Ok(mut s) = LOG.lock() {
        s.unseen = 0;
    }
}

pub fn clear() {
    if let Ok(mut s) = LOG.lock() {
        s.entries.clear();
        s.unseen = 0;
        s.version += 1;
    }
}

/// `at` as local `HH:MM:SS`.
pub fn clock(at: u64) -> String {
    #[cfg(unix)]
    {
        let t = at as libc::time_t;
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        // SAFETY: localtime_r only writes the struct it is given.
        if !unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
            return format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec);
        }
    }
    let d = at % 86_400;
    format!("{:02}:{:02}:{:02}", d / 3600, d / 60 % 60, d % 60)
}

/// A log record's module path as a short source: `varstotas::ui::library` → `library`,
/// `varstotas_kernel::io` → `kernel::io`.
fn source_of(target: &str) -> String {
    if let Some(rest) = target.strip_prefix("varstotas::") {
        return rest.rsplit("::").next().unwrap_or(rest).to_string();
    }
    target.strip_prefix("varstotas_").unwrap_or(target).to_string()
}

/// env_logger for the terminal plus the warnings and errors for the log window.
struct Logger {
    env: env_logger::Logger,
}

impl log::Log for Logger {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        self.env.enabled(meta) || keep(meta)
    }

    fn log(&self, record: &log::Record) {
        if self.env.enabled(record.metadata()) {
            self.env.log(record);
        }
        if keep(record.metadata()) {
            let level = if record.level() == log::Level::Error { Level::Error } else { Level::Warn };
            push(level, &source_of(record.target()), &record.args().to_string());
        }
    }

    fn flush(&self) {
        self.env.flush();
    }
}

/// Errors of any crate, warnings of Varstotas's own.
fn keep(meta: &log::Metadata) -> bool {
    meta.level() == log::Level::Error || (meta.level() == log::Level::Warn && meta.target().starts_with("varstotas"))
}

/// Install the logger (instead of `env_logger::init()`).
pub fn init() {
    let env = env_logger::Builder::from_default_env().build();
    let max = env.filter().max(log::LevelFilter::Warn);
    if log::set_boxed_logger(Box::new(Logger { env })).is_ok() {
        log::set_max_level(max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_count_up_and_sources_are_short() {
        // (other tests may log at the same time: only this source is looked at)
        error("test-import", "bad file");
        error("test-import", "bad file");
        push(Level::Warn, "test-import", "bad file");
        let (_, all) = snapshot();
        let e: Vec<_> = all.iter().filter(|e| e.source == "test-import").collect();
        assert_eq!(e.len(), 2);
        assert_eq!((e[0].count, e[0].level, e[1].level), (2, Level::Error, Level::Warn));
        assert!(unseen() >= 2);
        mark_seen();
        assert_eq!(source_of("varstotas::ui::library"), "library");
        assert_eq!(source_of("varstotas_kernel::io"), "kernel::io");
        assert_eq!(clock(0).len(), 8);
    }
}
