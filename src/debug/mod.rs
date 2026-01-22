pub mod debug_info;

#[cfg(feature = "trace")]
use common::utils::debug::runtime_storage::RuntimeStorage;

#[cfg(feature = "trace")]
use lazy_static::lazy_static;

#[cfg(feature = "trace")]
pub mod runtime_profiler;

#[cfg(feature = "trace")]
pub mod runtime_reporter;

#[cfg(feature = "trace")]
lazy_static! {
    pub static ref STORAGE: std::sync::Mutex<RuntimeStorage> = std::sync::Mutex::new(RuntimeStorage::new());
    pub static ref PROFILER: runtime_profiler::RuntimeProfiler = runtime_profiler::RuntimeProfiler;
}
