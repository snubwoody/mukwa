// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

#[derive(Debug, Default)]
pub struct BarChartSegment {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    value: f32,
}

pub struct BarChart {
    // The (x,y) position is (0,0) on the Cartesian axis.
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    series: Vec<f32>,
    /// The space between segments.
    spacing: f32,
}

impl BarChart {
    /// Create a new `BarChart`.
    ///
    /// ## Panics
    /// Panics if `width < 0` or `height < 0`.
    pub fn new(width: f32, height: f32, series: Vec<f32>) -> BarChart {
        assert!(width >= 0.0);
        assert!(height >= 0.0);

        BarChart {
            x: 0.0,
            y: 0.0,
            width,
            height,
            series,
            spacing: 50.0,
        }
    }

    pub fn segments(&self) -> Vec<BarChartSegment> {
        assert!(self.spacing >= 0.0);
        if self.series.is_empty() {
            return Vec::new();
        }

        let len = self.series.len();
        let bar_width = (self.width / len as f32) - (self.spacing * (len - 1) as f32);

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

        let mut segments = vec![];
        let mut x_offset = self.x;
        let spacing = 10.0;
        let baseline = self.y + 50.0;
        let top = baseline + 50.0;

        for value in &self.series {
            let height = map_value(*value, [min, max], [baseline, top]);
            let segment = BarChartSegment {
                x: x_offset,
                y: self.y,
                height,
                width: bar_width,
                value: *value,
            };
            segments.push(segment);

            x_offset += bar_width + spacing;
        }
        segments
    }

    pub fn draw(&self, pixmap: &mut Pixmap) {
        // TODO: add top and bottom baseline
        // Flip the y-axis
        let transform = Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, pixmap.height() as f32);
        for segment in self.segments() {
            dbg!(&segment);
            let rect =
                Rect::from_xywh(segment.x, segment.y, segment.width, segment.height).unwrap();
            let mut paint = Paint::default();
            paint.set_color(Color::BLACK);
            pixmap.fill_rect(rect, &paint, transform, None);
        }
    }
}

fn map_value(mut value: f32, input_range: [f32; 2], output_range: [f32; 2]) -> f32 {
    if value > input_range[1] {
        value = input_range[1]
    } else if value < input_range[0] {
        value = input_range[0]
    }

    let input_span = input_range[1] - input_range[0];
    if input_span == 0.0 {
        return output_range[0];
    }

    let scale = (output_range[1] - output_range[0]) / input_span;
    output_range[0] + (value - input_range[0]) * scale
}

#[cfg(test)]
mod test {
    use proptest::property_test;

    use super::*;

    #[test]
    fn test_map_value() {
        assert_eq!(map_value(1.0, [0.0, 1.0], [0.0, 100.0]), 100.0);
        assert_eq!(map_value(0.25, [0.0, 1.0], [0.0, 100.0]), 25.0);
    }

    #[test]
    fn draw_bar_chart() {
        let mut pixmap = Pixmap::new(500, 500).unwrap();
        pixmap.fill(Color::WHITE);

        let series = vec![24.24, 50.0];
        let chart = BarChart::new(500.0, 500.0, series);
        chart.draw(&mut pixmap);
        std::fs::create_dir_all("temp").unwrap();
        pixmap.save_png("temp/bar-chart.png").unwrap();
    }

    #[property_test]
    fn prop_segments_start_at_y_axis(
        series: Vec<f32>,
        #[strategy = 0f32..10000f32] x: f32,
        #[strategy = 0f32..10000f32] y: f32,
        #[strategy = 0f32..10000f32] width: f32,
        #[strategy = 0f32..10000f32] height: f32,
    ) {
        let mut chart = BarChart::new(width, height, series);
        chart.x = x;
        chart.y = y;
        for segment in chart.segments() {
            assert_eq!(segment.y, chart.y);
        }
    }

    #[property_test]
    fn prop_segments_width_fits_in_chart(
        series: Vec<f32>,
        #[strategy = 0f32..10000f32] width: f32,
        #[strategy = 0f32..10000f32] height: f32,
    ) {
        let chart = BarChart::new(width, height, series);
        let segments = chart.segments();
        let space_between = (segments.len() - 1) as f32 * chart.spacing;
        let total_width: f32 = chart.segments().iter().map(|s| s.width).sum();

        assert!(total_width + space_between <= chart.width);
    }

    #[test]
    #[should_panic]
    fn bar_chart_panics_on_negative_size() {
        BarChart::new(-2.0, -5.0, vec![]);
    }

    #[test]
    fn test_bar_chart() {
        let series = vec![24.24, 50.0];
        let chart = BarChart::new(500.0, 500.0, series);
        dbg!(chart.segments());
    }
}
