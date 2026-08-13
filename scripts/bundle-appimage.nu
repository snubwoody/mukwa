if not ("bin/linuxdeploy-x86_64.AppImage" | path exists) {
    print "Installing linuxdeploy..."
    wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
    mkdir bin/
    mv linuxdeploy-x86_64.AppImage bin/
}

let architecture = if $env.ARCHITECTURE == "aarch64" {
    "aarch64"
} else {
    "x86_64"
}

let target_triple = ($architecture) + "-unknown-linux-gnu"

print $"Building ($architecture) AppImage"

#if $architecture == "aarch64" {
#    cross build -r --target ($target_triple)
#} else {
#}

cargo build -r --target ($target_triple)

./bin/linuxdeploy-($architecture).AppImage --appdir build/AppDir -e target/($target_triple)/release/mukwa -d resources/mukwa.desktop --output appimage -i resources/icons/app-icon.svg
mv Mukwa-($architecture).AppImage build/
print "Successfully built AppImage"