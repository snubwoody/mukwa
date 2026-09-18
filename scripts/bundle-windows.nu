def main [architecture: string] {
    let app_version = open Cargo.toml | get workspace.package.version

    print $"Building ($architecture) Windows executable"
    let target_triple = ($architecture)-pc-windows-msvc

    let architectures_allowed = if $architecture == "x86_64" {
        "x64compatible"
    } else {
        "arm64"
    }

    cargo build --package mukwa --release --target $target_triple

    let mukwa_dir = "crates/mukwa"
    let resource_dir = ($env.TEMP)/MukwaBundleDir
    mkdir $resource_dir
    cp LICENSE $resource_dir
    cp target/($target_triple)/release/mukwa.exe $resource_dir
    cp ($mukwa_dir)/resources/icons/app-icon.ico $resource_dir

    iscc ($mukwa_dir)/resources/Mukwa.iss /DArchitecturesAllowed=($architectures_allowed) /DAppVersion=($app_version) /DResourceDir=($resource_dir) /Otarget/bundle/($target_triple) /FMukwa-($architecture)-Setup

    rm -rf $resource_dir
}

