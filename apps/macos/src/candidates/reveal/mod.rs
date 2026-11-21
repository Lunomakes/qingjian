//! 窗口展开 / 收起的时间线：从一个窗口矩形按缓动曲线走到另一个。

mod easing;

use std::time::{Duration, Instant};

use objc2_foundation::{NSPoint, NSRect, NSSize};

pub use easing::Easing;

/// 展开与收起都是 220 ms。
const DURATION: Duration = Duration::from_millis(220);

pub struct Reveal {
    from: NSRect,

    to: NSRect,

    started: Instant,

    easing: Easing,
}

impl Reveal {
    pub fn new(from: NSRect, to: NSRect, easing: Easing) -> Self {
        Self {
            from,
            to,
            started: Instant::now(),
            easing,
        }
    }

    /// `now` 时窗口该在的矩形（原点与大小一起线性插值），以及是否已走完。
    pub fn frame_at(&self, now: Instant) -> (NSRect, bool) {
        let elapsed = now.saturating_duration_since(self.started);
        let finished = elapsed >= DURATION;
        if finished {
            return (self.to, true);
        }
        let k = self
            .easing
            .value(elapsed.as_secs_f64() / DURATION.as_secs_f64());
        let lerp = |a: f64, b: f64| a + (b - a) * k;
        let rect = NSRect::new(
            NSPoint::new(
                lerp(self.from.origin.x, self.to.origin.x),
                lerp(self.from.origin.y, self.to.origin.y),
            ),
            NSSize::new(
                lerp(self.from.size.width, self.to.size.width),
                lerp(self.from.size.height, self.to.size.height),
            ),
        );
        (rect, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_endpoints_and_monotonic() {
        for easing in [Easing::EaseInOut, Easing::EaseOut] {
            assert!(easing.value(0.0).abs() < 1e-6);
            assert!((easing.value(1.0) - 1.0).abs() < 1e-6);
            let mut last = 0.0;
            for i in 1..=100 {
                let v = easing.value(f64::from(i) / 100.0);
                assert!(v >= last);
                last = v;
            }
        }
        assert!((Easing::EaseInOut.value(0.5) - 0.5).abs() < 1e-3);
        assert!(Easing::EaseOut.value(0.5) > 0.5);
    }

    #[test]
    fn frame_interpolates_and_finishes() {
        let from = NSRect::new(NSPoint::new(0.0, 100.0), NSSize::new(100.0, 40.0));
        let to = NSRect::new(NSPoint::new(0.0, 20.0), NSSize::new(200.0, 120.0));
        let reveal = Reveal::new(from, to, Easing::EaseInOut);
        let (rect, done) = reveal.frame_at(reveal.started + DURATION / 2);
        assert!(!done);
        assert!((rect.size.width - 150.0).abs() < 0.5);
        assert!((rect.origin.y - 60.0).abs() < 0.5);
        let (rect, done) = reveal.frame_at(reveal.started + DURATION);
        assert!(done && rect.size.height == 120.0);
    }
}
