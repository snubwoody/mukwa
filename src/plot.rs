// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::f32::consts::PI;
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Transform};

pub struct PieChart {
    x: f32,
    y: f32,
    series: Vec<f32>,
    total: f32,
    radius: f32,
    hole_radius: f32,
}

impl PieChart {
    pub fn new(x: f32, y: f32, series: Vec<f32>, radius: f32) -> Self {
        // TODO: test radius 0
        let total = series.iter().sum();

        Self {
            series,
            total,
            x,
            y,
            radius,
            hole_radius: 0.0,
        }
    }

    /// Sets the inner hole radius.
    ///
    /// ## Panics
    /// Panics if hole radius is `>=` the pie chart radius.
    pub fn set_hole_radius(&mut self, radius: f32) {
        assert!(radius < self.radius);
        self.hole_radius = radius;
    }

    fn draw_slice(&self, start_angle: f32, slice: f32) -> Path {
        let ratio = slice / self.total;
        let end_theta = 2.0 * PI * ratio;

        let inner_start = theta_to_ordinal_coord(self.hole_radius, start_angle, (self.x, self.y));
        let outer_start = theta_to_ordinal_coord(self.radius, start_angle, (self.x, self.y));
        let inner_end =
            theta_to_ordinal_coord(self.hole_radius, start_angle + end_theta, (self.x, self.y));

        let mut pb = PathBuilder::new();
        // Draw the start line
        pb.move_to(inner_start.0, inner_start.1);
        pb.line_to(outer_start.0, outer_start.1);

        // Draw the outer arc
        draw_arc(
            &mut pb,
            self.radius,
            start_angle,
            end_theta,
            (self.x, self.y),
        );

        // Draw the end line
        pb.line_to(inner_end.0, inner_end.1);

        // Draw the inner arc
        draw_arc(
            &mut pb,
            self.hole_radius,
            start_angle,
            end_theta,
            (self.x, self.y),
        );

        pb.close();
        pb.finish().unwrap()
    }

    /// Draws the pie chart onto the Pixmap
    pub fn draw(&self, pixmap: &mut Pixmap) {
        let colors = [
            Color::from_rgba8(61, 144, 255, 255),
            Color::from_rgba8(216, 226, 255, 255),
            Color::from_rgba8(0, 70, 138, 255),
            Color::from_rgba8(127, 171, 255, 255),
            Color::from_rgba8(174, 198, 255, 255),
            Color::from_rgba8(1, 117, 222, 255),
            Color::from_rgba8(236, 241, 255, 255),
        ];
        let mut start_angle = 0.0;

        for (index, slice) in self.series.iter().enumerate() {
            let ratio = slice / self.total;
            let end_theta = 2.0 * PI * ratio;

            let path = self.draw_slice(start_angle, *slice);
            let mut paint = Paint::default();
            paint.set_color(colors[index % colors.len()]);
            pixmap.fill_path(
                &path,
                &paint,
                FillRule::EvenOdd,
                Transform::identity(),
                None,
            );
            start_angle += end_theta;
        }
    }
}

/// Draws an arc approximated using quadratic beziers, starting at `start_angle` and
/// sweeping by `sweep_angle`.
fn draw_arc(
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
        let control = theta_to_ordinal_coord(control_radius, mid_angle, center);
        let end = theta_to_ordinal_coord(radius, end_angle, center);
        pb.quad_to(control.0, control.1, end.0, end.1);

        current_angle = end_angle;
    }
}

fn theta_to_ordinal_coord(radius: f32, theta: f32, center: (f32, f32)) -> (f32, f32) {
    // polar coordinates are (r, theta)
    // convert to (x, y) coord, with center as offset

    let (sin, cos) = theta.sin_cos();
    (
        radius * cos + center.0, // x
        radius * sin + center.1, // y
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn draw_arc_splits_into_45deg_segments() {
        let mut pb = PathBuilder::new();
        draw_arc(&mut pb, 50.0, 0.0, PI / 4.0, (0.0, 0.0));
        assert_eq!(pb.len(), 2);
    }

    #[test]
    fn test_theta_to_ordinal_coord() {
        let test_coord = |theta: f32, expected: (f32, f32)| {
            let (x, y) = theta_to_ordinal_coord(50.0, theta, (50.0, 50.0));
            assert_eq!(x.round(), expected.0);
            assert_eq!(y.round(), expected.1);
        };

        test_coord(0.0, (100.0, 50.0));
        test_coord(2.0 * PI, (100.0, 50.0));
        test_coord(PI, (0.0, 50.0));
    }

    #[test]
    fn draw_pie_chart() {
        let series: Vec<f32> = vec![20.0, 24.0, 100.0];
        let mut pie = PieChart::new(250.0, 250.0, series, 50.0);
        pie.hole_radius = 10.0;

        let mut pixmap = Pixmap::new(500, 500).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);
        pie.draw(&mut pixmap);
        pixmap.save_png("temp/image.png").unwrap();
    }
}
