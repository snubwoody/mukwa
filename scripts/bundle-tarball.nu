let target_triple = "x86_64-unknown-linux-gnu"
let bundle_name = $"mukwa-x86_64"
let bundle_dir = $"build/($bundle_name)"
print $"Bundling (ansi green)x86_64(ansi reset) Linux (ansi purple).tar.gz(ansi reset)"

cargo build -p mukwa -r --target ($target_triple)

let dirs = [
    $"($bundle_dir)"
    $"($bundle_dir)/bin"
    $"($bundle_dir)/share"
    $"($bundle_dir)/share/applications"
    $"($bundle_dir)/share/metainfo"
    $"($bundle_dir)/share/icons"
    $"($bundle_dir)/share/icons/hicolor"
    $"($bundle_dir)/share/icons/hicolor/scalable"
]

print ""
for $dir in $dirs {
    print $"Creating directory (ansi purple)($dir)(ansi reset)"
    mkdir $dir
}
print ""

def copy [origin: string, destination: string] {
    print $"Copying ($origin) to (ansi purple)($destination)(ansi reset)"
    cp $origin $destination
}

let resource_dir = "crates/mukwa/resources"
copy target/($target_triple)/release/mukwa ($bundle_dir)/bin
copy ($resource_dir)/mukwa.desktop ($bundle_dir)/share/applications/com.wakunguma.Mukwa.desktop
copy ($resource_dir)/mukwa.metainfo.xml ($bundle_dir)/share/metainfo/com.wakunguma.Mukwa.metainfo.xml
copy ($resource_dir)/icons/app-icon.svg ($bundle_dir)/share/icons/hicolor/scalable/mukwa.svg
copy ($resource_dir)/tar/Makefile ($bundle_dir)/
copy ($resource_dir)/tar/README.md ($bundle_dir)/
copy LICENSE ($bundle_dir)

print "\nCreating tarball"
mkdir target/bundle/($target_triple)
tar -czvf target/bundle/($target_triple)/mukwa-x86_64.tar.gz -C build $bundle_name
