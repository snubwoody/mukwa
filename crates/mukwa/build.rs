// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

fn main() {
    unsafe {
        std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1");
    }

    slint_build::compile("ui/app.slint").unwrap();

    #[cfg(target_os = "windows")]
    {
        use winresource::WindowsResource;

        println!("cargo:rerun-if-changed=resources");
        let mut rc = WindowsResource::new();
        rc.set_icon("resources/icons/app-icon.ico");
        rc.set("ProductName", "Mukwa");
        rc.set("FileDescription", "Mukwa");
        rc.set("LegalCopyright", "Copyright © 2026 Wakunguma Kalimukwa");
        rc.set("CompanyName", "Wakunguma Kalimukwa");
        rc.set_manifest_file("resources/manifest.xml");
        rc.compile().unwrap()
    }
}
