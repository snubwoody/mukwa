if not ("bin/linuxdeploy-x86_64.AppImage" | path exists) {
    echo Installing linuxdeploy...
    wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
    mkdir bin/
    mv linuxdeploy-x86_64.AppImage bin/
}

let target_triple = "x86_64-unknown-linux-gnu"

cargo build -r --target x86_64-unknown-linux-gnu
./bin/linuxdeploy-x86_64.AppImage --appdir build/AppDir -e target/($target_triple)/release/mukwa -d resources/mukwa.desktop --output appimage -i resources/icons/app-icon.svg
mv Mukwa-x86_64.AppImage build/