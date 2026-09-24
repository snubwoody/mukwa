def main [architecture: string] {
    let app_version = open Cargo.toml | get workspace.package.version
    let target_triple = ($architecture) + "-apple-darwin"
    let mukwa_dir = "crates/mukwa"
    let bundle_dir = $"target/bundle/($target_triple)"
    let app_dir = $"($bundle_dir)/Mukwa.app"

    let dirs = [
        $"($app_dir)"
        $"($app_dir)/Contents"
        $"($app_dir)/Contents/MacOS"
        $"($app_dir)/Contents/Resources"
    ]

    print $"Building ($architecture) macOS executable"
    cargo build --target $target_triple --package mukwa --release
    let mukwa_dir = "crates/mukwa"

    for $dir in $dirs {
        print $"Creating directory (ansi purple)($dir)(ansi reset)"
        mkdir $dir
    }

    print "\nDeploying files\n"
    copy ($mukwa_dir)/resources/Info.plist ($app_dir)/Contents
    copy ($mukwa_dir)/resources/icons/app-icon.icns ($app_dir)/Contents/Resources/AppIcon.icns
    copy LICENSE ($app_dir)/Contents/Resources
    copy target/($target_triple)/release/mukwa ($app_dir)/Contents/MacOS
    hdiutil create -srcFolder $app_dir -o ($bundle_dir)/Mukwa-($app_version)-($architecture).dmg
}

def copy [origin: string, destination: string] {
    print $"Copying ($origin) to (ansi purple)($destination)(ansi reset)"
    cp $origin $destination
}