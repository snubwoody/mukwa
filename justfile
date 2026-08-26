set windows-shell := ["powershell", "-c"]

add-migration name:
    cargo run --bin utils migrate new {{ name }}

migrate:
    cargo run --bin utils migrate up

# Run the app
run:
    cargo run --bin mukwa

lint:
    cargo clippy --all-targets

bundle-windows:
    scripts/bundle-windows.ps1
