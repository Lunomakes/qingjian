//! 把位图贴进比它小的视图（窗口正在展开 / 收起）：内容从顶部露出、在底边截断，窗口的圆角与边框保持完整。

use objc2::rc::Retained;
use objc2_app_kit::{NSBezierPath, NSCompositingOperation, NSImage, NSWindingRule};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::super::bounds::ViewBounds;

/// 圆角、边框与抗锯齿一共占的宽度（点），加在投影留边上就是要原样保留的角块边长。
const CORNER: f64 = 18.0;

/// 边框连同抗锯齿的宽度（点）：只有这一圈从位图的底边与右边取，里面的内容不跟着贴到底边。
const EDGE: f64 = 2.0;

/// 在翻转视图里把 `image`（尺寸 `bounds.size`）画进 `view_size`。`radius` 是窗口的圆角（点），
/// 没有系统材质时为 `None`，只做保角拼接。
pub(super) fn draw(
    image: &Retained<NSImage>,
    bounds: &ViewBounds,
    view_size: NSSize,
    radius: Option<f64>,
) {
    let size = bounds.size;
    if view_size.width >= size.width && view_size.height >= size.height {
        draw_piece(image, rect(0.0, 0.0, size.width, size.height), NSRect::ZERO);
        return;
    }
    let width = view_size.width.min(size.width);
    let height = view_size.height.min(size.height);
    let Some(radius) = radius else {
        draw_corners(image, bounds, width, height);
        return;
    };
    let content = bounds.content;
    let right = size.width - content.origin.x - content.size.width;
    let top = size.height - content.origin.y - content.size.height;
    let window = rect(
        content.origin.x,
        top,
        width - content.origin.x - right,
        height - top - content.origin.y,
    );
    // 两次裁剪叠加：先按窗口圆角贴顶部对齐的整张位图，再缩到边框那一圈补上底边与右边
    let outline = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(window, radius, radius);
    outline.addClip();
    draw_piece(image, rect(0.0, 0.0, size.width, size.height), NSRect::ZERO);
    let inner = NSRect::new(
        NSPoint::new(window.origin.x + EDGE, window.origin.y + EDGE),
        NSSize::new(
            (window.size.width - EDGE * 2.0).max(0.0),
            (window.size.height - EDGE * 2.0).max(0.0),
        ),
    );
    let ring = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(window, radius, radius);
    ring.appendBezierPath(&NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        inner,
        (radius - EDGE).max(0.0),
        (radius - EDGE).max(0.0),
    ));
    ring.setWindingRule(NSWindingRule::EvenOdd);
    ring.addClip();
    draw_corners(image, bounds, width, height);
}

/// 取四块拼成 `width × height`：左上主体、右侧与底部两条边、右下角块。
fn draw_corners(image: &Retained<NSImage>, bounds: &ViewBounds, width: f64, height: f64) {
    let size = bounds.size;
    let margin = bounds.content.origin.x.max(bounds.content.origin.y);
    let c = (margin + CORNER).min(width / 2.0).min(height / 2.0);
    let (body_w, body_h) = (width - c, height - c);
    let top_src = size.height - body_h;
    // (目的矩形（左上原点）, 来源矩形（位图坐标，左下原点）)
    let pieces = [
        (
            rect(0.0, 0.0, body_w, body_h),
            rect(0.0, top_src, body_w, body_h),
        ),
        (
            rect(body_w, 0.0, c, body_h),
            rect(size.width - c, top_src, c, body_h),
        ),
        (rect(0.0, body_h, body_w, c), rect(0.0, 0.0, body_w, c)),
        (rect(body_w, body_h, c, c), rect(size.width - c, 0.0, c, c)),
    ];
    for (to, from) in pieces {
        draw_piece(image, to, from);
    }
}

/// `from` 为空矩形时取整张位图。
fn draw_piece(image: &Retained<NSImage>, to: NSRect, from: NSRect) {
    // SAFETY: hints 传 None，其余参数都是普通值；在 drawRect: 内调用，有当前图形上下文。
    unsafe {
        image.drawInRect_fromRect_operation_fraction_respectFlipped_hints(
            to,
            from,
            NSCompositingOperation::Copy,
            1.0,
            true,
            None,
        );
    }
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
}
