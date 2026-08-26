if not ("bin/linuxdeploy-x86_64.AppImage" | path exists) {
    print "Installing linuxdeploy..."
    wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
    mkdir bin/
    mv linuxdeploy-x86_64.AppImage bin/
}

let architecture = if $env.ARCHITECTURE? == "aarch64" {
    "aarch64"
} else {
    "x86_64"
}

let target_triple = ($architecture) + "-unknown-linux-gnu"
let mukwa_dir = "crates/mukwa"

print $"Building ($architecture) AppImage"

#if $architecture == "aarch64" {
#    cross build -r --target ($target_triple)
#} else {
#}

cargo build -p mukwa -r --target ($target_triple)

mkdir build/AppDir/usr/share/metainfo
cp ($mukwa_dir)/resources/mukwa.metainfo.xml build/AppDir/usr/share/metainfo/com.wakunguma.Mukwa.appdata.xml
cp ($mukwa_dir)/resources/mukwa.desktop build/com.wakunguma.Mukwa.desktop
cp ($mukwa_dir)/resources/icons/app-icon.svg build/mukwa.svg
cp target/($target_triple)/release/mukwa build/mukwa

./bin/linuxdeploy-($architecture).AppImage --appdir build/AppDir -e build/mukwa -d build/com.wakunguma.Mukwa.desktop --output appimage -i build/mukwa.svg

mv Mukwa-($architecture).AppImage build/
print "Successfully built AppImage"
