//! Disposable Z0 behavior proof, not a production API decision.

use crate::{
    AppContext as _, Bounds, Context, InputEvent as _, InteractiveElement as _, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement as _, Pixels, Render, ScrollHandle,
    ScrollWheelEvent, StatefulInteractiveElement as _, Styled as _, TestAppContext, Window, div,
    point, px, rgb, size,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct ZoomProbe {
    bounds: ScrollHandle,
    hits: Rc<Cell<usize>>,
    wheel: Rc<RefCell<Option<crate::Point<Pixels>>>>,
}

impl Render for ZoomProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let hits = self.hits.clone();
        let wheel = self.wheel.clone();
        div().size_full().child(
            div()
                .id("zoom-target")
                .size(px(100.))
                .bg(rgb(0x123456))
                .track_scroll(&self.bounds)
                .on_mouse_down(MouseButton::Left, move |_, _, _| hits.set(hits.get() + 1))
                .on_scroll_wheel(move |event, _, _| *wheel.borrow_mut() = Some(event.position)),
        )
    }
}

#[test]
fn window_zoom_coordinate_roundtrip() {
    let mut cx = TestAppContext::single();
    let bounds = ScrollHandle::new();
    let hits = Rc::new(Cell::new(0));
    let wheel = Rc::new(RefCell::new(None));
    let handle = cx.add_window({
        let bounds = bounds.clone();
        let hits = hits.clone();
        let wheel = wheel.clone();
        move |_, _| ZoomProbe {
            bounds,
            hits,
            wheel,
        }
    });
    cx.test_window(handle.into())
        .simulate_scale_factor_change(1.);
    cx.update_window(handle.into(), |_, window, cx| {
        window.resize(size(px(1000.), px(1000.)));
        window.bounds_changed(cx);
        window.set_zoom(0.9);
        window.draw(cx).clear(cx);
        assert_eq!(
            bounds.bounds(),
            Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)))
        );
        let viewport: f32 = window.viewport_size().width.into();
        assert!(
            (viewport - 1000. / 0.9).abs() <= 1.,
            "zoom viewport must be design pixels, got {viewport}"
        );
        assert!((window.scale_factor() - 0.9).abs() < 0.001);
        assert!(
            window.painted_quads().iter().any(|quad| {
                (quad.bounds.size.width.0 - 90.).abs() <= 0.5
                    && (quad.bounds.size.height.0 - 90.).abs() <= 0.5
            }),
            "the 100 design-pixel div must paint at 90 physical pixels"
        );
        for (physical, expected_hits) in [(45., 1), (95., 1)] {
            window.dispatch_event(
                MouseDownEvent {
                    position: point(px(physical), px(physical)),
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(hits.get(), expected_hits, "physical {physical} hit/miss");
        }
        window.dispatch_event(
            ScrollWheelEvent {
                position: point(px(45.), px(45.)),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        let wheel = wheel.borrow().expect("wheel handler must run");
        assert!((f32::from(wheel.x) - 50.).abs() <= 0.5);
        assert!((f32::from(wheel.y) - 50.).abs() <= 0.5);
        window.dispatch_event(
            MouseMoveEvent {
                position: point(px(90.), px(90.)),
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        assert!((f32::from(window.mouse_position().x) - 100.).abs() <= 0.5);
        assert!((f32::from(window.mouse_position().y) - 100.).abs() <= 0.5);
        window.bounds_changed(cx);
        assert!(
            (window.scale_factor() - 0.9).abs() < 0.001,
            "resize must preserve zoom"
        );
        assert!((f32::from(window.viewport_size().width) - 1000. / 0.9).abs() <= 1.);
        window.set_zoom(0.1);
        assert!((window.scale_factor() - 0.5).abs() < 0.001);
        window.set_zoom(3.);
        assert!((window.scale_factor() - 2.).abs() < 0.001);
        window.set_zoom(f32::NAN);
        assert_eq!(window.scale_factor(), 1.);
    })
    .expect("test window update");
}
