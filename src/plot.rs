// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::f32::consts::PI;
use tiny_skia::{
    Color, FillRule, Paint, Path, PathBuilder, PathSegment, Pixmap, Point, Stroke, Transform,
};

pub struct PieSegment {
    // The center position
    x: f32,
    y: f32,
    ratio: f32,
    color: Color,
    start_angle: f32,
    hole_radius: f32,
    radius: f32,
    label_line_length: f32,
    label: String,
}

impl PieSegment {
    /// Draws the pie chart segment onto the pixmap.
    pub fn draw(&self, pixmap: &mut Pixmap) {
        let path = self.arc_path();
        let mut paint = Paint::default();
        paint.set_color(self.color);
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::EvenOdd,
            Transform::identity(),
            None,
        );
    }

    /// Draws the segment's label onto the pixmap.
    pub fn draw_labels(&self, pixmap: &mut Pixmap) {
        let path = self.label_line_path();
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(100, 100, 100, 255));
        let stroke = Stroke::default();
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    fn label_line_path(&self) -> Path {
        let end_theta = 2.0 * PI * self.ratio;
        let mid_angle = self.start_angle + end_theta * 0.5;

        let start = theta_to_ordinal_coord(self.radius, mid_angle, (self.x, self.y));
        let end = theta_to_ordinal_coord(
            self.radius + self.label_line_length,
            mid_angle,
            (self.x, self.y),
        );

        let mut pb = PathBuilder::new();
        pb.move_to(start.0, start.1);
        pb.line_to(end.0, end.1);
        pb.finish().unwrap()
    }

    pub fn label_position(&self) -> (f32, f32) {
        let end_theta = 2.0 * PI * self.ratio;
        let mid_angle = self.start_angle + end_theta * 0.5;
        let approx_font_size = 16.0;

        theta_to_ordinal_coord(
            self.radius + self.label_line_length + approx_font_size,
            mid_angle,
            (self.x, self.y),
        )
    }

    fn segment_to_svg(&self, segment: &PathSegment) -> String {
        match segment {
            PathSegment::MoveTo(Point { x, y }) => format!("M {x} {y} "),
            PathSegment::LineTo(Point { x, y }) => format!("L {x} {y} "),
            PathSegment::QuadTo(Point { x: x1, y: y1 }, Point { x, y }) => {
                format!("Q {x1} {y1}, {x} {y} ")
            }
            PathSegment::CubicTo(
                Point { x: x2, y: y2 },
                Point { x: x1, y: y1 },
                Point { x, y },
            ) => format!("C {x2} {y2}, {x1} {y1}, {x} {y} "),
            PathSegment::Close => String::from("Z"),
        }
    }

    /// Generates an SVG path string for the arc path.
    pub fn arc_svg(&self) -> String {
        // TODO: start_position function
        let mut svg = String::new();
        let path = self.arc_path();
        for segment in path.segments() {
            svg += &self.segment_to_svg(&segment);
        }
        svg
    }

    /// Generates an SVG path string for the label line path.
    pub fn label_line_svg(&self) -> String {
        let mut svg = String::new();
        let path = self.label_line_path();
        for segment in path.segments() {
            svg += &self.segment_to_svg(&segment);
        }
        svg
    }

    fn arc_path(&self) -> Path {
        let end_theta = 2.0 * PI * self.ratio;
        let start_angle = self.start_angle;

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
            start_angle + end_theta,
            -end_theta,
            (self.x, self.y),
        );

        pb.close();
        pb.finish().unwrap()
    }
}

pub struct PieChart {
    x: f32,
    y: f32,
    series: Vec<f32>,
    total: f32,
    radius: f32,
    hole_radius: f32,
    colors: Vec<Color>,
    label_line_length: f32,
    labels: Vec<String>,
}

impl PieChart {
    /// Creates a new pie chart centered at `x`,`y`.
    pub fn new(x: f32, y: f32, series: Vec<f32>, radius: f32) -> Self {
        let total = series.iter().sum();
        let colors = vec![
            Color::from_rgba8(61, 144, 255, 255),
            Color::from_rgba8(216, 226, 255, 255),
            Color::from_rgba8(0, 70, 138, 255),
            Color::from_rgba8(127, 171, 255, 255),
            Color::from_rgba8(174, 198, 255, 255),
            Color::from_rgba8(1, 117, 222, 255),
            Color::from_rgba8(236, 241, 255, 255),
        ];

        Self {
            series,
            total,
            x,
            y,
            radius,
            hole_radius: 0.0,
            colors,
            label_line_length: 0.0,
            labels: Vec::new(),
        }
    }

    /// Sets the inner hole radius.
    ///
    /// ## Panics
    /// Panics if hole radius is `>=` the pie chart radius.
    pub fn with_hole_radius(mut self, radius: f32) -> Self {
        assert!(radius < self.radius);
        self.hole_radius = radius;
        self
    }

    /// Sets the pie chart colors.
    pub fn with_colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn with_label_line_length(mut self, length: f32) -> Self {
        self.label_line_length = length;
        self
    }

    /// Generates the pie chart segments.
    pub fn segments(&self) -> Vec<PieSegment> {
        let mut start_angle = 0.0;
        let mut segments = vec![];
        for (index, slice) in self.series.iter().enumerate() {
            let ratio = slice / self.total;
            let end_theta = 2.0 * PI * ratio;
            let segment = PieSegment {
                x: self.x,
                y: self.y,
                hole_radius: self.hole_radius,
                ratio,
                color: self.colors[index % self.colors.len()],
                start_angle,
                radius: self.radius,
                label_line_length: self.label_line_length,
                label: self
                    .labels
                    .get(index)
                    .map(|s| s.to_owned())
                    .unwrap_or_default(),
            };

            start_angle += end_theta;
            segments.push(segment);
        }
        segments
    }

    /// Draws the pie chart onto the Pixmap.
    pub fn draw(&self, pixmap: &mut Pixmap) {
        for segment in self.segments() {
            segment.draw(pixmap);
        }
    }

    /// Draws the pie chart, with labels, onto the Pixmap.
    pub fn draw_with_labels(&self, pixmap: &mut Pixmap) {
        for segment in self.segments() {
            segment.draw(pixmap);
            segment.draw_labels(pixmap);
        }
    }
}

/// Draws a circular arc approximated using quadratic beziers, starting at `start_angle` and
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
}
