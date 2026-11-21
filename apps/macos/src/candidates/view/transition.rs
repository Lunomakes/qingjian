//! 候选窗在单行与矩阵之间展开 / 收起时的窗口动画：跟着定时器逐跳改窗口矩形，内容用保角裁剪露出来。

use std::time::{Duration, Instant};

use objc2::{DefinedClass, MainThreadMarker};
use objc2_foundation::NSRect;

use super::CandidateView;
use crate::candidates::bitmap::reduce_motion;
use crate::candidates::frame::Frame;
use crate::candidates::reveal::{Easing, Reveal};

/// 窗口动画的跳动间隔，与渲染器的过渡一致。
const TICK: Duration = Duration::from_millis(16);

impl CandidateView {
    pub fn set_animations(&self, enabled: bool) {
        self.ivars().animations.set(enabled);
    }

    /// 新一帧到来前调：取消还在播的窗口动画；若这一帧是单行与矩阵之间的切换就记下方向，
    /// 收起时把旧矩阵位图留住。必须在位图渲染器画新一帧之前调。
    pub(super) fn plan_reveal(&self, frame: &Frame, visible: bool) {
        self.cancel_reveal();
        self.ivars().transition.set(None);
        let previous = self.grid_state(&self.ivars().frame.borrow());
        let next = self.grid_state(frame);
        let (Some(previous), Some(next)) = (previous, next) else {
            return;
        };
        if previous == next || !visible || !self.ivars().animations.get() || reduce_motion() {
            return;
        }
        if next {
            self.ivars().transition.set(Some(Easing::EaseInOut));
        } else {
            self.ivars().transition.set(Some(Easing::EaseOut));
            if let Some(bitmap) = &mut *self.ivars().bitmap.borrow_mut() {
                bitmap.hold_current();
            }
        }
    }

    /// 窗口从 `from` 走到 `to`；这一帧不是切换时返回 `false`，窗口直接落在 `to`。
    pub fn start_reveal(&self, from: NSRect, to: NSRect) -> bool {
        let Some(easing) = self.ivars().transition.take() else {
            return false;
        };
        *self.ivars().reveal.borrow_mut() = Some(Reveal::new(from, to, easing));
        self.sync_animation();
        true
    }

    /// 动画走一步：改窗口矩形并重画；走完落在终点矩形、放掉旧位图。
    pub(super) fn step_reveal(&self) {
        let step = self
            .ivars()
            .reveal
            .borrow()
            .as_ref()
            .map(|reveal| reveal.frame_at(Instant::now()));
        let Some((rect, finished)) = step else {
            return;
        };
        if finished {
            self.cancel_reveal();
        }
        if let Some(window) = self.window() {
            window.setFrame_display(rect, true);
            window.invalidateShadow();
        }
        self.setNeedsDisplay(true);
    }

    pub(super) fn cancel_reveal(&self) {
        if self.ivars().reveal.borrow_mut().take().is_none() {
            return;
        }
        if let Some(bitmap) = &mut *self.ivars().bitmap.borrow_mut() {
            bitmap.release_held();
        }
    }

    /// 渲染器的动画与窗口动画里要得更勤的那个间隔；都没有为 `None`。
    pub(super) fn animation_interval(&self) -> Option<Duration> {
        let renderer = self
            .ivars()
            .bitmap
            .borrow()
            .as_ref()
            .and_then(|bitmap| bitmap.next_frame());
        let reveal = self.ivars().reveal.borrow().is_some().then_some(TICK);
        [renderer, reveal].into_iter().flatten().min()
    }

    /// 这一帧是位图候选窗时它是不是矩阵；符号面板与系统绘制为 `None`。
    fn grid_state(&self, frame: &Frame) -> Option<bool> {
        let bitmap = self.ivars().bitmap.borrow().is_some();
        (bitmap && !frame.is_panel()).then_some(frame.columns > 0)
    }

    pub(super) fn timer_target(&self) -> Option<(Duration, MainThreadMarker)> {
        Some((self.animation_interval()?, MainThreadMarker::new()?))
    }
}
