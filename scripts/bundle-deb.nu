let bundle_dir = "target/bundle/deb/Mukwa-x86_64"
let app_version = open Cargo.toml | get workspace.package.version
let target_triple = "x86_64-unknown-linux-gnu"

let dirs = [
    $"($bundle_dir)"
    $"($bundle_dir)/DEBIAN"
    $"($bundle_dir)/usr/bin"
    $"($bundle_dir)/usr/share"
    $"($bundle_dir)/usr/share/applications"
    $"($bundle_dir)/usr/share/metainfo"
    $"($bundle_dir)/usr/share/icons/hicolor/scalable/apps"
]

for $dir in $dirs {
    print $"Creating directory (ansi purple)($dir)(ansi reset)"
    mkdir $dir
}

cargo build --target ($target_triple) --release
install target/($target_triple)/release/mukwa ($bundle_dir)/usr/bin/ -m 0755

cp crates/mukwa/resources/mukwa.desktop ($bundle_dir)/usr/share/applications
cp crates/mukwa/resources/icons/app-icon.svg ($bundle_dir)/usr/share/icons/hicolor/scalable/apps/mukwa.svg

let size = (du ($bundle_dir)/usr | get physical.0) / 1Kib | math round
print "Creating control file"
touch ($bundle_dir)/DEBIAN/control
$"Package: mukwa
Version: ($app_version)
Section: utils
Standards-Version: 4.6.2
Priority: optional
Architecture: amd64
Installed-Size: ($size)
Depends: zenity, libxkbcommon-x11-0, libfontconfig1, wl-clipboard, libx11-6, libxcursor1
Homepage: https://github.com/snubwoody/mukwa
Maintainer: Wakunguma Kalimukwa <contact@wakunguma.com>
Description: Mukwa a free, privacy focused personal finance app, designed to help you track your spending and manage your budgets.
"
| save -f ($bundle_dir)/DEBIAN/control

dpkg-deb --build $bundle_dir
