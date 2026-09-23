let target_triple = "x86_64-unknown-linux-gnu"
let version = open Cargo.toml | get workspace.package.version

print $"Bundling (ansi green)x86_64(ansi reset) Linux (ansi purple).tar.gz(ansi reset)"

cargo build -p mukwa -r --target ($target_triple)

let dirs = [
    "build"
    "build/lib"
    "build/share"
    "build/share/applications"
    "build/share/metainfo"
    "build/share/icons"
    "build/share/icons/hicolor"
    "build/share/icons/hicolor/scalable"
]

print ""
for $dir in $dirs {
    print $"Creating directory (ansi purple)($dir)(ansi reset)"
    mkdir $dir
}
print ""

def copy [origin: string, destination: string] {
    print $"Copying (ansi green)($origin)(ansi reset) to (ansi purple)($destination)(ansi reset)"
    cp $origin $destination
}

let resource_dir = "crates/mukwa/resources"
copy target/($target_triple)/release/mukwa build/bin
copy ($resource_dir)/mukwa.desktop build/share/applications/com.wakunguma.Mukwa.desktop
copy ($resource_dir)/mukwa.metainfo.xml build/share/metainfo/com.wakunguma.Mukwa.metainfo.xml
copy ($resource_dir)/icons/app-icon.svg build/share/icons/hicolor/scalable/mukwa.svg
copy ($resource_dir)/tar/Makefile build/
copy ($resource_dir)/tar/README.md build/
copy LICENSE build

print "\nCreating tarball"
mkdir target/bundle/($target_triple)
tar -czvf target/bundle/($target_triple)/mukwa-v($version)-x86_64.tar.gz -C build .
