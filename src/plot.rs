// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

fn pie_chart() {
    let mut pixmap = Pixmap::new(500, 500).unwrap();
    pixmap.fill(tiny_skia::Color::WHITE);

    let x = 50.0;
    let width = 200.0;
    let height = 20.0;
    let y = 250.0;

    let mut pb = PathBuilder::new();
    pb.move_to(x, y);
    pb.quad_to(x + width * 0.5, 0.0, x + width, y);
    pb.line_to(x + width - 20.0, y + height);
    pb.quad_to(x + width * 0.5, 0.0, x + 20.0, y + height);
    pb.line_to(x, y);
    pb.close();
    let path = pb.finish().unwrap();

    let stroke = Stroke::default();
    let mut paint = Paint::default();
    paint.set_color(tiny_skia::Color::BLACK);

    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    // pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
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
