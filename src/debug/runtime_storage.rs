use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use lazy_static::lazy_static;

use super::runtime_reporter::RuntimeReporter;

pub(crate) type SpansType = HashMap<&'static str, (Duration, u32)>;
pub(crate) type LastType = HashMap<&'static str, Duration>;

pub struct RuntimeStorage {
    spans: SpansType,
    last: LastType,
}

impl RuntimeStorage {
    pub fn new() -> Self {
        Self {
            spans: HashMap::new(),
            last: HashMap::new(),
        }
    }

    pub fn push(&mut self, name: &'static str, elapsed: Duration) {
        let entry = self.spans.entry(name).or_insert((Duration::ZERO, 0));
        entry.0 += elapsed;
        entry.1 += 1;

        self.last.insert(name, elapsed);
    }

    pub fn flush(&mut self) {
        if self.spans.is_empty() && self.last.is_empty() {
            return;
        }

        let clear = RuntimeReporter::report(&self.spans, &self.last);

        if clear {
            self.spans.clear();
            self.last.clear();
        }
    }
}

lazy_static! {
    pub static ref RUNTIME_STORAGE: Mutex<RuntimeStorage> = Mutex::new(RuntimeStorage::new());
}
