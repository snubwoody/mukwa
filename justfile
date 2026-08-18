set windows-shell := ["powershell", "-c"]

add-migration name:
    cargo run --bin cli migrate new {{ name }}

migrate:
    cargo run --bin cli migrate up

format-slint:
    slint-lsp format ui/**/*.slint --inline

bundle-windows:
    scripts/bundle-windows.ps1
