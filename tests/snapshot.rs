// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use mukwa::Result;
use mukwa::plot::PieChart;
use std::path::Path;
use tiny_skia::{Color, Pixmap};

fn test_snapshot(path: impl AsRef<Path>, pixmap: &Pixmap) {
    let pixels = pixmap.data();
    let image = image::open(path).unwrap();
    let image_bytes = image.as_bytes();
    assert_eq!(pixels, image_bytes);
}

#[test]
fn simple_pie_chart() -> Result<()> {
    let colors = vec![
        Color::from_rgba8(100, 24, 24, 255),
        Color::from_rgba8(0, 254, 24, 255),
        Color::from_rgba8(0, 24, 254, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![20.0, 24.0, 100.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/simple_pie_chart.png", &pixmap);
    Ok(())
}

#[test]
fn zero_value_slice() -> Result<()> {
    let colors = vec![
        Color::from_rgba8(100, 24, 24, 255),
        Color::from_rgba8(0, 254, 24, 255),
        Color::from_rgba8(0, 24, 254, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![20.0, 0.0, 100.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/zero_value_slice.png", &pixmap);
    Ok(())
}

#[test]
fn very_thin_donut_chart() -> Result<()> {
    let colors = vec![
        Color::from_rgba8(100, 24, 24, 255),
        Color::from_rgba8(0, 254, 24, 255),
        Color::from_rgba8(0, 24, 254, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![20.0, 0.0, 100.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);
    pie.set_hole_radius(99.5);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/very_thin_donut_chart.png", &pixmap);
    Ok(())
}

#[test]
fn simple_donut_chart() -> Result<()> {
    let colors = vec![
        Color::from_rgba8(100, 24, 24, 255),
        Color::from_rgba8(0, 254, 24, 255),
        Color::from_rgba8(0, 24, 254, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![20.0, 24.0, 100.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);
    pie.set_hole_radius(50.0);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/simple_donut_chart.png", &pixmap);
    Ok(())
}

#[test]
fn single_slice() -> Result<()> {
    // Make sure a single slice isn't somehow broken
    let colors = vec![Color::from_rgba8(0, 0, 255, 255)];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![100.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/single_slice.png", &pixmap);
    Ok(())
}

#[test]
fn color_wrap_around() -> Result<()> {
    // Make sure the colors wrap around if there are more slices than colors
    let colors = vec![
        Color::from_rgba8(0, 0, 255, 255),
        Color::from_rgba8(255, 0, 255, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![100.0, 500.0, 90.0, 200.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/color_wrap_around.png", &pixmap);
    Ok(())
}

#[test]
fn tiny_slice() -> Result<()> {
    // Test that tiny slices are basically invisible and don't produce artifacts
    let colors = vec![
        Color::from_rgba8(25, 150, 255, 255),
        Color::from_rgba8(242, 0, 255, 255),
        Color::from_rgba8(90, 0, 5, 255),
        Color::from_rgba8(24, 150, 202, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![0.0001, 0.1, 90.0, 200.0];
    let mut pie = PieChart::new(center, center, series, 100.0);
    pie.set_colors(colors);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/tiny_slice.png", &pixmap);
    Ok(())
}

#[test]
fn clip() -> Result<()> {
    let colors = vec![
        Color::from_rgba8(25, 150, 255, 255),
        Color::from_rgba8(242, 0, 255, 255),
    ];

    let size = 250.0;
    let center = size / 2.0;
    let series: Vec<f32> = vec![90.0, 200.0];
    let mut pie = PieChart::new(250.0, center, series, 100.0);
    pie.set_colors(colors);

    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    pixmap.fill(Color::WHITE);
    pie.draw(&mut pixmap);

    test_snapshot("tests/references/clip.png", &pixmap);
    Ok(())
}
