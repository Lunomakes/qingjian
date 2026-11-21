//! 动画缓动曲线：CSS 同款三次贝塞尔，起终点固定在 (0,0) 与 (1,1)。

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// cubic-bezier(0.42, 0, 0.58, 1)：展开用。
    EaseInOut,

    /// cubic-bezier(0, 0, 0.58, 1)：收起用。
    EaseOut,
}

impl Easing {
    /// 时间进度 `t`（0–1）对应的动画进度。
    pub fn value(self, t: f64) -> f64 {
        let (x1, y1, x2, y2) = match self {
            Self::EaseInOut => (0.42, 0.0, 0.58, 1.0),
            Self::EaseOut => (0.0, 0.0, 0.58, 1.0),
        };
        let t = t.clamp(0.0, 1.0);
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..32 {
            let mid = (low + high) / 2.0;
            if bezier(mid, x1, x2) < t {
                low = mid;
            } else {
                high = mid;
            }
        }
        bezier((low + high) / 2.0, y1, y2)
    }
}

/// 一维三次贝塞尔，控制点 `(0, p1, p2, 1)`。
fn bezier(s: f64, p1: f64, p2: f64) -> f64 {
    let u = 1.0 - s;
    3.0 * u * u * s * p1 + 3.0 * u * s * s * p2 + s * s * s
}
