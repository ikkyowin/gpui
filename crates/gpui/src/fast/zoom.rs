//! Disposable whole-window zoom at GPUI's platform boundary.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

use crate::{Bounds, Edges, FileDropEvent, Pixels, PlatformInput, ScrollDelta, Window};
use std::ops::MulAssign;

pub(crate) fn from_platform<T: MulAssign<f32>>(mut value: T, zoom: f32) -> T {
    value *= zoom.recip();
    value
}

pub(crate) fn to_platform<T: MulAssign<f32>>(mut value: T, zoom: f32) -> T {
    value *= zoom;
    value
}

impl Window {
    /// Sets whole-window design-pixel zoom independently of the display scale.
    pub fn set_zoom(&mut self, zoom: f32) {
        let zoom = if zoom.is_nan() {
            1.0
        } else {
            zoom.clamp(0.5, 2.0)
        };
        if zoom == self.zoom {
            return;
        }
        self.mouse_position = from_platform(to_platform(self.mouse_position, self.zoom), zoom);
        self.zoom = zoom;
        self.viewport_size = from_platform(self.platform_window.content_size(), zoom);
        if let Some(inset) = self.client_inset() {
            self.set_client_inset(inset);
        }
        self.refresh();
    }

    pub(crate) fn zoom_input_region(&self, region: Option<&[Bounds<Pixels>]>) {
        let region = region.map(|bounds| {
            bounds
                .iter()
                .map(|b| to_platform(*b, self.zoom))
                .collect::<Vec<_>>()
        });
        self.platform_window.set_input_region(region.as_deref());
    }
}

pub(crate) fn exclusive_zone(zone: Pixels, zoom: f32) -> Pixels {
    if zone > Pixels::ZERO {
        to_platform(zone, zoom)
    } else {
        zone
    }
}

pub(crate) fn design_insets(insets: Edges<Pixels>, zoom: f32) -> Edges<Pixels> {
    Edges {
        top: from_platform(insets.top, zoom),
        right: from_platform(insets.right, zoom),
        bottom: from_platform(insets.bottom, zoom),
        left: from_platform(insets.left, zoom),
    }
}

pub(crate) fn design_event(mut event: PlatformInput, zoom: f32) -> PlatformInput {
    match &mut event {
        PlatformInput::MouseMove(e) => e.position = from_platform(e.position, zoom),
        PlatformInput::MouseDown(e) => e.position = from_platform(e.position, zoom),
        PlatformInput::MouseUp(e) => e.position = from_platform(e.position, zoom),
        PlatformInput::MousePressure(e) => e.position = from_platform(e.position, zoom),
        PlatformInput::MouseExited(e) => e.position = from_platform(e.position, zoom),
        PlatformInput::ScrollWheel(e) => {
            e.position = from_platform(e.position, zoom);
            if let ScrollDelta::Pixels(delta) = &mut e.delta {
                *delta = from_platform(*delta, zoom);
            }
        }
        PlatformInput::Pinch(e) => e.position = from_platform(e.position, zoom),
        PlatformInput::Touch(e) => {
            e.position = from_platform(e.position, zoom);
            e.predicted_position = e.predicted_position.map(|p| from_platform(p, zoom));
        }
        PlatformInput::FileDrop(
            FileDropEvent::Entered { position, .. }
            | FileDropEvent::Pending { position }
            | FileDropEvent::Submit { position },
        ) => *position = from_platform(*position, zoom),
        _ => {}
    }
    event
}
