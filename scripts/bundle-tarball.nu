let target_triple = "x86_64-unknown-linux-gnu"
let version = open Cargo.toml | get workspace.package.version

print $"Bundling (ansi green)x86_64(ansi reset) Linux (ansi purple).tar.gz(ansi reset)"

cargo build -p mukwa -r --target ($target_triple)

let dirs = [
    "build"
    "build/bin"
    "build/lib"
    "build/share"
    "build/share/applications"
    "build/share/icons"
    "build/share/icons/hicolor"
    "build/share/icons/hicolor/scalable"
]

print ""
for $dir in $dirs {
    print $"Creating directory ($dir)"
    mkdir $dir
}
print ""

cp target/($target_triple)/release/mukwa build/bin
cp crates/mukwa/resources/mukwa.desktop build/share/applications/com.wakunguma.Mukwa.desktop
cp crates/mukwa/resources/icons/app-icon.svg build/share/icons/hicolor/scalable/mukwa.svg
cp LICENSE build

let excluded_libs = [
    "libz",
    "libc",
    "libstdc++",
    "libm",
    "libgcc_s",
    "libbz2",
    "libexpat"
]

let libs = ldd build/bin/mukwa
| lines
| str trim
| where { |it| $excluded_libs | all {|el| not ($it | str contains $el)}} # Filter excluded libraries
| parse "{name} => {path} ({addr})"

for $lib in $libs {
    print $"Copying (ansi purple)($lib.path)(ansi reset) to build/lib"
    cp $lib.path build/lib
}

print ""
tar -czvf mukwa-v($version)-linux-x86_64.tar.gz -C build .
