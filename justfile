[windows]
set shell := ["powershell", "-c"]

# Create a new migration
[working-directory("crates/core")]
add-migration name:
    cargo run -p utils migrate new {{ name }}

migrate:
    cargo run -p utils migrate up

# Run the app
run:
    cargo run --bin mukwa

lint:
    cargo clippy --all-targets

bundle-windows:
    scripts/bundle-windows.ps1
