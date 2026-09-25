//! Input-latency instrumentation — measurement only, no behavior changes.
//!
//! Two counters exposed as process-wide statics so the embedding app can
//! drain them into its own periodic metrics log:
//!
//! - [`INPUT_QUEUE_AGE`]: how long a pressed-button mouse-move or scroll
//!   NSEvent sat between the OS generating it and gpui dispatching it
//!   (recorded in `handle_view_event`).
//! - [`GPU_PRESENT`]: wall time from entering `MetalRenderer::draw` (including
//!   any `next_drawable` wait) to the command buffer's GPU completion.
//!
//! Recording sites run on the main thread and Metal's completion thread;
//! draining is lock-free and resets the counter.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub struct LatencyCounter {
    sum_ns: AtomicU64,
    max_ns: AtomicU64,
    count: AtomicU64,
}

impl LatencyCounter {
    pub const fn new() -> Self {
        Self {
            sum_ns: AtomicU64::new(0),
            max_ns: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    pub fn record(&self, d: Duration) {
        let ns = d.as_nanos() as u64;
        self.sum_ns.fetch_add(ns, Ordering::Relaxed);
        self.max_ns.fetch_max(ns, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns `(sum, max, count)` since the last drain and resets to zero.
    pub fn drain(&self) -> (Duration, Duration, u64) {
        let sum = self.sum_ns.swap(0, Ordering::Relaxed);
        let max = self.max_ns.swap(0, Ordering::Relaxed);
        let count = self.count.swap(0, Ordering::Relaxed);
        (Duration::from_nanos(sum), Duration::from_nanos(max), count)
    }
}

pub static INPUT_QUEUE_AGE: LatencyCounter = LatencyCounter::new();
pub static GPU_PRESENT: LatencyCounter = LatencyCounter::new();
