let bundle_dir = "target/bundle/flatpak"

def main [command: string] {
    match $command {
        "help" => {
            help
        }
        "build" => {
            build
        }
        "install" => {
            install
        }
    }
}

# Build the flatpak
def "main build" [] {
    if not ("bin/flatpak-cargo-generator.py" | path exists) {
        print "Installing flatpak-cargo-generator.py from https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo"
        mkdir bin/
        wget https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/refs/heads/master/cargo/flatpak-cargo-generator.py -O bin/flatpak-cargo-generator.py
    }

    if not (".venv" | path exists) {
        uv venv --clear
        uv pip install --python .venv/bin/python aiohttp
        uv pip install --python .venv/bin/python tomlkit
    }

    uv run python bin/flatpak-cargo-generator.py Cargo.lock -o ($bundle_dir)/cargo-sources.json
    print "Building flatpak..."
    mkdir $bundle_dir
    cp crates/mukwa/resources/com.wakunguma.Mukwa.yaml $bundle_dir
    flatpak-builder --force-clean build ($bundle_dir)/com.wakunguma.Mukwa.yaml
}

# Install the flatpak
def "main install" [] {
    flatpak-builder --force-clean --user --repo=repo --install build ($bundle_dir)/com.wakunguma.Mukwa.yaml
}