use godot::classes::performance::Monitor;
use godot::classes::{Engine, Performance};
use godot::obj::Singleton;
use std::borrow::Cow;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::debug::format_grouped_lines::format_grouped_lines;

use super::runtime_storage::{LastType, SpansType};

const REPORT_LIMIT: usize = 10;
const REPORT_COOLDOWN: Duration = Duration::from_secs(10);

macro_rules! lags_template {
    () => {
        "&cLags detected! ({fps} fps):&r
&cGodot:&r
{godot}
&cProcess {process:.1}ms:&r
{lines}"
    };
}

pub struct RuntimeReporter;

static LAST_REPORT: Mutex<Option<Instant>> = Mutex::new(None);

fn godot_stats() -> String {
    
    let p = Performance::singleton();

    let process = p.get_monitor(Monitor::TIME_PROCESS);
    let physics = p.get_monitor(Monitor::TIME_PHYSICS_PROCESS);
    let navigation = p.get_monitor(Monitor::TIME_NAVIGATION_PROCESS);

    let draw_calls = p.get_monitor(Monitor::RENDER_TOTAL_DRAW_CALLS_IN_FRAME);
    let objects = p.get_monitor(Monitor::RENDER_TOTAL_OBJECTS_IN_FRAME);
    let primitives = p.get_monitor(Monitor::RENDER_TOTAL_PRIMITIVES_IN_FRAME);

    let mem_static = p.get_monitor(Monitor::MEMORY_STATIC) / 1024.0 / 1024.0;
    let vram = p.get_monitor(Monitor::RENDER_VIDEO_MEM_USED) / 1024.0 / 1024.0;

    format!(
        "  &acpu process &7{:.1}ms
  &aphysics &7{:.1}ms
  &anavigation &7{:.1}ms
  &arender draws &7{:.0}
  &aobjects &7{:.0}
  &aprimitives &7{:.0}
  &amemory ram &7{:.1}MB
  &avram &7{:.1}MB",
        process * 1000.0,
        physics * 1000.0,
        navigation * 1000.0,
        draw_calls,
        objects,
        primitives,
        mem_static,
        vram,
    )
}

impl RuntimeReporter {
    pub fn report(spans: &SpansType, last: &LastType) -> bool {
        {
            let mut last_report = LAST_REPORT.lock().unwrap();
            if let Some(t) = *last_report {
                if t.elapsed() < REPORT_COOLDOWN {
                    return false;
                }
            }
            *last_report = Some(Instant::now());
        }

        let mut items: Vec<(&Cow<'static, str>, Duration, Duration)> = spans
            .iter()
            .map(|(name, (total, count))| {
                let avg = *total / *count;
                let last = *last.get(name).unwrap_or(&Duration::ZERO);
                (name, last, avg)
            })
            .collect();

        let fps = Engine::singleton().get_frames_per_second();
        if fps >= 60.0 {
            return false;
        }

        items.sort_by(|a, b| b.1.cmp(&a.1));

        let process = Performance::singleton().get_monitor(Monitor::TIME_PROCESS);

        let msg = format!(
            lags_template!(),
            godot = godot_stats(),
            process = process * 1000.0,
            lines = format_grouped_lines(items, REPORT_LIMIT),
            fps = fps,
        );

        log::warn!(target: "frame", "{}", msg);
        true
    }
}
