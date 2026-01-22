pub mod debug_info;

#[cfg(debug_assertions)]
use common::utils::debug::runtime_storage::RuntimeStorage;

#[cfg(debug_assertions)]
use lazy_static::lazy_static;

#[cfg(debug_assertions)]
pub mod runtime_profiler;

#[cfg(debug_assertions)]
pub mod runtime_reporter;

#[macro_export]
macro_rules! span {
    ($name:expr) => {{
        #[cfg(debug_assertions)]
        {
            if $crate::debug::debug_info::DebugInfo::is_active() {
                Some($crate::debug::runtime_profiler::RuntimeSpan::new($name))
            } else {
                None
            }
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }};
}

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref STORAGE: std::sync::Mutex<RuntimeStorage> = std::sync::Mutex::new(RuntimeStorage::new());
}
