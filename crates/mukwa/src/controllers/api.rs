// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use crate::ui::ComboBoxItem;
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::fmt::CurrencyFormatter;
use native_dialog::DialogBuilder;
use slint::{ComponentHandle, DataTransfer, Model, ModelRc, ToSharedString, VecModel};
use std::str::FromStr;
use tracing::{info, warn};

pub fn bind(window: &ui::MainWindow) {
    let api = window.global::<ui::Api>();

    // FIXME: panics on unequal lengths
    api.on_csv_combobox_options(|records| {
        let mut index = 0;
        let mut combobox_items = vec![];
        if let Some(record) = records.iter().next() {
            for (index, cell) in record.iter().enumerate() {
                let item = ComboBoxItem {
                    text: cell.clone(),
                    value: index.to_shared_string(),
                };
                combobox_items.push(item);
            }
        }
        ModelRc::new(VecModel::from(combobox_items))
    });

    api.on_read_csv(|data| {
        if !data.has_file_paths() {
            warn!("No csv files were chosen");
            return ModelRc::new(VecModel::default());
        }

        let path = data.file_paths().unwrap().next().unwrap();
        let mut reader = csv::Reader::from_path(path).unwrap();
        // let mut records = vec![];
        let model = VecModel::default();
        for result in reader.records() {
            let record = result.unwrap();
            let cells: Vec<_> = record.iter().map(|s| s.to_shared_string()).collect();
            let inner_model = VecModel::from(cells);
            model.push(ModelRc::new(inner_model));
        }
        ModelRc::new(model)
    });

    api.on_open_csv(|| {
        let result = DialogBuilder::file()
            .add_filter("CSV file", ["csv"])
            .open_single_file()
            .show()
            .unwrap();
        match result {
            Some(path) => {
                info!("Opened csv file at {:?}", path);
                let mut data = DataTransfer::default();
                data.set_file_paths([path]);
                data
            }
            None => {
                warn!("No csv file found");
                DataTransfer::default()
            }
        }
    });

    api.on_format_money_without_symbol({
        move |value| {
            // Empty strings represent null values
            if value.is_empty() {
                return value;
            }

            let formatter = CurrencyFormatter::new();
            match Money::from_str(&value) {
                Ok(value) => formatter
                    .format_money_without_symbol(value)
                    .to_shared_string(),
                Err(err) => {
                    warn!("Error parsing Money: {err}");
                    formatter
                        .format_money_without_symbol(Money::ZERO)
                        .to_shared_string()
                }
            }
        }
    });

    api.on_window_size({
        let window = window.as_weak();
        move || {
            if let Some(window) = window.upgrade() {
                let window = window.window();
                let size = window.size().to_logical(window.scale_factor());
                return (size.height, size.width);
            }
            warn!("Empty window");
            (0.0, 0.0)
        }
    });

    api.on_window_position({
        let window = window.as_weak();
        move || {
            if let Some(window) = window.upgrade() {
                let window = window.window();
                let pos = window.position().to_logical(window.scale_factor());
                return (pos.x, pos.y);
            }
            warn!("Empty window");
            (0.0, 0.0)
        }
    });

    api.on_parse_date(|date| {
        let date = Date::strptime("%Y-%m-%d", &date)
            .inspect_err(|err| warn!("{err}"))
            .unwrap_or(Zoned::now().date());

        ui::Date {
            year: date.year() as i32,
            month: date.month() as i32,
            day: date.day() as i32,
        }
    });

    api.on_today(|| Zoned::now().date().to_shared_string());
    api.on_money_to_float(|money| {
        Money::from_str(&money)
            .inspect_err(|err| warn!("{err}"))
            .unwrap_or_default()
            .inner() as f32
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ui::MainWindow;

    #[test]
    fn read_csv_return_default_if_empty() -> crate::Result<()> {
        let window = MainWindow::new()?;
        bind(&window);
        let api = window.global::<ui::Api>();
        let model = api.invoke_read_csv(DataTransfer::default());
        assert_eq!(model.iter().len(), 0);
        Ok(())
    }
}
