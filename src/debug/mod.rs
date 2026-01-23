pub mod debug_info;

#[cfg(debug_assertions)]
use common::utils::debug::runtime_storage::RuntimeStorage;

#[cfg(debug_assertions)]
use lazy_static::lazy_static;

#[cfg(debug_assertions)]
pub mod runtime_profiler;

#[cfg(debug_assertions)]
pub mod runtime_reporter;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! span {
    ($name:expr) => {
        $crate::debug::debug_info::DebugInfo::is_active()
            .then(|| $crate::debug::runtime_profiler::RuntimeSpan::new($name))
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! span {
    ($name:expr) => {
        None::<$crate::debug::runtime_profiler::RuntimeSpan>
    };
}

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref STORAGE: std::sync::Mutex<RuntimeStorage> = std::sync::Mutex::new(RuntimeStorage::new());
}
