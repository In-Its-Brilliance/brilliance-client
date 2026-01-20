use godot::prelude::*;
mod client_scripts;
mod console;
mod controller;
mod debug;
mod entities;
mod logger;
mod network;
mod scenes;
mod ui;
mod utils;
mod world;

struct Brilliance;

pub const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;
pub const WARNING_TIME: std::time::Duration = std::time::Duration::from_millis(15);
pub const MAX_THREADS: usize = 12;

#[cfg(feature = "trace")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

#[gdextension]
unsafe impl ExtensionLibrary for Brilliance {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            if let Err(e) = log::set_logger(&logger::CONSOLE_LOGGER) {
                log::error!(target: "main", "log::set_logger error: {}", e)
            }
            log::set_max_level(LOG_LEVEL);
        }
    }
}
