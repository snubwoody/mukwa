// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::f32::consts::PI;
use tiny_skia::{FillRule, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

const KAPPA: f32 = 0.55228474983079;

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

        let circ_radius = 50.0;
        let arc_length = 100.0;

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

pub struct PieChart {
    x: f32,
    y: f32,
    series: Vec<f32>,
    total: f32,
    radius: f32,
    hole_radius: f32,
}

impl PieChart {
    fn new() -> Self {
        let series: Vec<f32> = vec![20.0, 24.0, 100.0];
        let total = series.iter().sum();

        Self {
            series,
            total,
            x: 250.0,
            y: 250.0,
            radius: 50.0,
            hole_radius: 10.0,
        }
    }

    /// Draws an arc approximated using quadratic beziers, starting at `start_angle` and
    /// sweeping by `sweep_angle`.
    fn draw_arc(
        &self,
        pb: &mut PathBuilder,
        radius: f32,
        start_angle: f32,
        sweep_angle: f32,
        center: (f32, f32),
    ) {
        let max_step = std::f32::consts::FRAC_PI_4;
        let steps = (sweep_angle.abs() / max_step).ceil() as usize;
        let step_angle = sweep_angle / steps as f32;

        let mut current_angle = start_angle;

        for _ in 0..steps {
            let half_step = step_angle / 2.0;
            let mid_angle = current_angle + half_step;
            let end_angle = current_angle + step_angle;

            let control_radius = radius / half_step.cos();
            let start = theta_to_ordinal_coord(radius, current_angle, center);
            let control = theta_to_ordinal_coord(control_radius, mid_angle, center);
            let end = theta_to_ordinal_coord(radius, end_angle, center);
            pb.move_to(start.0, start.1);
            pb.quad_to(control.0, control.1, end.0, end.1);

            current_angle = end_angle;
        }
    }

    fn draw_slice(&self, start_angle: f32, slice: f32) -> Path {
        let ratio = slice / self.total;
        let end_theta = 2.0 * PI * ratio;

        let inner_start = theta_to_ordinal_coord(self.hole_radius, start_angle, (self.x, self.y));
        let outer_start = theta_to_ordinal_coord(self.radius, start_angle, (self.x, self.y));

        let mut pb = PathBuilder::new();
        // TODO: need to draw end line and close the path
        pb.move_to(inner_start.0, inner_start.1);
        pb.line_to(outer_start.0, outer_start.1);

        self.draw_arc(
            &mut pb,
            self.radius,
            start_angle,
            end_theta,
            (self.x, self.y),
        );
        self.draw_arc(
            &mut pb,
            self.hole_radius,
            start_angle,
            end_theta,
            (self.x, self.y),
        );
        pb.finish().unwrap()
    }

    fn draw(&self) {
        let mut pixmap = Pixmap::new(500, 500).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        // This would be the start radian
        let mut start_angle = 0.0;

        let stroke = Stroke::default();
        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::BLACK);

        for slice in &self.series {
            let ratio = slice / self.total;
            let end_theta = 2.0 * PI * ratio;

            // TODO: maybe pass start angle as mutable
            let path = self.draw_slice(start_angle, *slice);
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            start_angle += end_theta;
        }

        pixmap.save_png("temp/image-2.png").unwrap();
    }
}

fn theta_to_ordinal_coord(radius: f32, theta: f32, ordinal_offset: (f32, f32)) -> (f32, f32) {
    // polar coordinates are (r, theta)
    // convert to (x, y) coord, with center as offset

    let (sin, cos) = theta.sin_cos();
    (
        radius * cos + ordinal_offset.0, // x
        radius * sin + ordinal_offset.1, // y
    )
}

fn pie_chart() {
    let mut pixmap = Pixmap::new(500, 500).unwrap();
    pixmap.fill(tiny_skia::Color::WHITE);

    let segment = Segment::xywh(50.0, 250.0, 200.0, 50.0).radius(100.0);
    segment.draw(&mut pixmap);
    let segment = Segment::xywh(250.0, 250.0, 200.0, 50.0).radius(100.0);
    segment.draw(&mut pixmap);
    pixmap.save_png("image.png").unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn draw_pie_chart() {
        let pie = PieChart::new();
        pie.draw();
    }
}
