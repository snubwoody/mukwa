use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

fn main() {
    let path = Path::new("ui");
    println!("cargo:rustc-env=SLINT_INCLUDE_PATH={}", path.display());
    let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);
    slint_build::compile_with_config("ui/app.slint", config).unwrap();

    #[cfg(target_os = "windows")]
    {
        use winresource::WindowsResource;

        let mut rc = WindowsResource::new();
        rc.set_icon("resources/icons/app-icon.ico");
        rc.set("ProductName", "Mukwa");
        rc.set("FileDescription", "Mukwa");
        rc.set("LegalCopyright", "Copyright © 2026 Wakunguma Kalimuwka");
        rc.set("CompanyName", "Wakunguma Kalimuwka");
        rc.compile().unwrap()
    }
}
