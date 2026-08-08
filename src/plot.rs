// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use tiny_skia::{FillRule, Paint, Path, PathBuilder, Pixmap, Transform};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct Segment {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
}

impl Segment {
    /// Creates a new `Segment`.
    fn xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Segment {
            x,
            y,
            width,
            height,
            radius: 50.0,
        }
    }

    /// Sets the `radius` of the segment.
    fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    fn to_path(&self) -> Path {
        let x = self.x;
        let width = self.width;
        let height = self.height;
        let y = self.y;

        let radius = self.radius;
        let cx = x + width * 0.5;
        let cy = y - radius;

        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.quad_to(cx, cy, x + width, y);
        pb.line_to(x + width - 20.0, y + height);
        pb.quad_to(cx, cy + height, x + 20.0, y + height);
        pb.line_to(x, y);
        pb.close();
        pb.finish().unwrap()
    }

    /// Draws the segment onto the pixmap.
    fn draw(&self, pixmap: &mut Pixmap) {
        let path = self.to_path();

        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::BLACK);

        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn pie_chart() {
    let mut pixmap = Pixmap::new(500, 500).unwrap();
    pixmap.fill(tiny_skia::Color::WHITE);

    let segment = Segment::xywh(50.0, 250.0, 200.0, 50.0).radius(100.0);
    segment.draw(&mut pixmap);
    pixmap.save_png("image.png").unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn draw_pie_chart() {
        pie_chart();
    }
}
