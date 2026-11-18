// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use tiny_skia::Pixmap;

fn draw_bar_chart(pixmap: &mut Pixmap, series: Vec<f32>) {}
#[cfg(test)]
mod test {
    use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

    use super::*;

    #[test]
    fn draw_bar_chart() {
        let mut pixmap = Pixmap::new(500, 500).unwrap();
        // FIXME: handle panics or return default
        let rect = Rect::from_xywh(20.0, 20.0, 200.0, 200.0).unwrap();
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        std::fs::create_dir_all("temp").unwrap();
        pixmap.save_png("temp/bar-chart.png").unwrap();
    }
}
