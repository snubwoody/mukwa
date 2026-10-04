let bundle_dir = "target/bundle/flatpak"
let resource_dir = "crates/mukwa/resources"

def main [] {

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
    cp ($resource_dir)/com.wakunguma.Mukwa.yaml $bundle_dir
    flatpak-builder --force-clean build ($bundle_dir)/com.wakunguma.Mukwa.yaml
}

# Install the flatpak
def "main install" [] {
    flatpak-builder --force-clean --user --repo=repo --install build ($bundle_dir)/com.wakunguma.Mukwa.yaml
}

# Run the flatpak linter against the manifest files
def "main lint" [] {
    print "Linting appstream..."
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream ($resource_dir)/mukwa.metainfo.xml
    print "Linting flatpak manifest..."
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest ($bundle_dir)/com.wakunguma.Mukwa.yaml
}
