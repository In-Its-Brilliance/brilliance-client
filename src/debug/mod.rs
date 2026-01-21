pub mod debug_info;

#[cfg(feature = "trace")]
use lazy_static::lazy_static;

#[cfg(feature = "trace")]
pub mod runtime_profiler;

#[cfg(feature = "trace")]
pub mod runtime_storage;

#[cfg(feature = "trace")]
pub mod runtime_reporter;

#[cfg(feature = "trace")]
pub mod format_grouped_lines;

#[cfg(feature = "trace")]
lazy_static! {
    pub static ref STORAGE: std::sync::Mutex<runtime_storage::RuntimeStorage> =
        std::sync::Mutex::new(runtime_storage::RuntimeStorage::new());

    pub static ref PROFILER: runtime_profiler::RuntimeProfiler = runtime_profiler::RuntimeProfiler;
}
