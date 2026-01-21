use std::time::Instant;

use lazy_static::lazy_static;

pub struct RuntimeSpan {
    profiler: &'static RuntimeProfiler,
    name: &'static str,
    start: Instant,
}

lazy_static! {
    pub static ref RUNTIME_PROFILER: RuntimeProfiler = RuntimeProfiler;
}

pub struct RuntimeProfiler;

impl RuntimeProfiler {
    pub fn span(&'static self, name: &'static str) -> RuntimeSpan {
        RuntimeSpan {
            profiler: self,
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for RuntimeSpan {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        crate::debug::runtime_storage::RUNTIME_STORAGE
            .lock()
            .unwrap()
            .push(self.name.clone(), elapsed);
    }
}
