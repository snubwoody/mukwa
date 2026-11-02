// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// TODO:
// - net profit line chart
// - how will credit payments interact with this

use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

pub struct LineChart {
    series: Vec<f32>,
    width: f32,
    height: f32,
}

// TODO: test empty series
impl LineChart {
    pub fn new(width: f32, height: f32, series: Vec<f32>) -> LineChart {
        LineChart {
            series,
            width,
            height,
        }
    }

    fn points(&self) -> Vec<Point> {
        let max = self
            .series
            .iter()
            .copied()
            .reduce(f32::max)
            .unwrap_or_default();
        let min = self
            .series
            .iter()
            .copied()
            .reduce(f32::min)
            .unwrap_or_default();

        let spacing = 20.0;
        let mut center_start = (self.width) / 2.0;

        let mut points = vec![];
        for point in &self.series {
            // Invert the y-axis to make coordinates start from the
            // bottom left
            let y = map(*point, [min, max], [self.height, 0.0]);
            let x = center_start;
            points.push(Point::new(x, y));
            center_start += spacing;
        }
        points
    }

    fn draw_2(&self, pixmap: &mut Pixmap) {
        // FIXME: panics at single point
        if self.series.len() == 1 {
            return;
        }

        let mut pb = PathBuilder::new();

        for (index, point) in self.points().iter().enumerate() {
            let x = point.x;
            let y = point.y;
            if index == 0 {
                pb.move_to(x, y);
            } else {
                pb.line_to(x, y);
            }
            dbg!(x, y);
        }

        let path = pb.finish().unwrap();
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);

        let mut stroke = Stroke::default();
        stroke.width = 2.0;
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    /// Draws the line chart onto the Pixmap.
    pub fn draw(&self, pixmap: &mut Pixmap) {
        // TODO: we also need to draw within the width.
        // TODO: draw circles at points
        // TODO: need to find the x-axis line if there's negative values
        // TODO: maybe curved and not curved
        // TODO: let padding = 0.2; let height = self.height - self.height * padding;
        let padding = 0.2;
        let min = self
            .series
            .iter()
            .copied()
            .reduce(f32::min)
            .unwrap_or_default();
        let max = self
            .series
            .iter()
            .copied()
            .reduce(f32::max)
            .unwrap_or_default();

        let min_y = 0.0 + padding * self.height;
        let max_y = self.height - padding * self.height;
        let mut x_offset = padding * self.width;
        let x_diff = self.width / self.series.len() as f32;
        // FIXME: flipped coords

        let mut pb = PathBuilder::new();
        // TODO: subtract padding from end?
        // TODO: this might be pretty bad for small series
        for (index, point) in self.series.iter().enumerate() {
            // TODO: confusing names
            // TODO: flip the coordinates
            let y = map(*point, [min, max], [min_y, max_y]);
            dbg!(y, min_y, max_y, min, max);
            if index == 0 {
                pb.move_to(x_offset, y);
            } else {
                pb.line_to(x_offset, y);
            }
            x_offset += x_diff;
        }

        let path = pb.finish().unwrap();

        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);

        let mut stroke = Stroke::default();
        stroke.width = 2.0;
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

/// Maps a value from one range to another. Any overflow or underflow is clipped to the min or
/// max respectively.
pub const fn map(value: f32, input_range: [f32; 2], output_range: [f32; 2]) -> f32 {
    let mut value = value;
    if value > input_range[1] {
        value = input_range[1]
    } else if value < input_range[0] {
        value = input_range[0]
    }

    let scale = (output_range[1] - output_range[0]) / (input_range[1] - input_range[0]);
    let offset = input_range[0] * (scale) + output_range[0];

    value * scale + offset
}

#[cfg(test)]
mod test {
    use super::*;
    use tiny_skia::{Color, Pixmap};

    #[test]
    fn scale_and_flip_points() {
        let size = 100.0;
        let series = vec![0.0, 0.5, 1.0];
        let chart = LineChart::new(size as f32, size as f32, series);
        let points = chart.points();
        assert_eq!(points[0].y, 100.0);
        assert_eq!(points[1].y, 50.0);
        assert_eq!(points[2].y, 0.0);
    }

    #[test]
    fn test() {
        let size = 500;
        let mut pixmap = Pixmap::new(size, size).unwrap();
        pixmap.fill(Color::WHITE);

        // let series = vec![20.0, 50.0, 200.0, 120.0];
        let series = vec![0.0,0.1, 0.5, 0.9, 1.0];
        let chart = LineChart::new(size as f32, size as f32, series);
        chart.draw_2(&mut pixmap);
        let path = "temp/image.png";
        pixmap.save_png(path).unwrap();
    }
}
