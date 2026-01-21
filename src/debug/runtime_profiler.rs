use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;

pub struct RuntimeProfiler {
    active: Mutex<HashMap<String, Instant>>,
}

pub struct RuntimeSpan {
    profiler: &'static RuntimeProfiler,
    name: String,
}

lazy_static! {
    pub static ref RUNTIME_PROFILER: RuntimeProfiler = RuntimeProfiler {
        active: Mutex::new(HashMap::new()),
    };
}

impl RuntimeProfiler {
    pub fn span<S: Into<String>>(&'static self, name: S) -> RuntimeSpan {
        let name = name.into();
        self.active.lock().unwrap().insert(name.clone(), Instant::now());
        RuntimeSpan { profiler: self, name }
    }

    fn finish(&self, name: &str) -> Option<Duration> {
        self.active.lock().unwrap().remove(name).map(|s| s.elapsed())
    }
}

impl Drop for RuntimeSpan {
    fn drop(&mut self) {
        if let Some(elapsed) = self.profiler.finish(&self.name) {
            crate::debug::runtime_storage::RUNTIME_STORAGE
                .lock()
                .unwrap()
                .push(self.name.clone(), elapsed);
        }
    }
}
