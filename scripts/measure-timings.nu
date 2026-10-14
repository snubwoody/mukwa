# cargo +nightly build

# let file_path = "crates/mukwa/ui/settings.slint"
# open $file_path | lines | update 50 "        text: @tr(\"Currenc\");" | str join (char newline) | save -f $file_path

cargo +nightly build -p mukwa
cargo +nightly clean -p mukwa
cargo +nightly build -p mukwa
