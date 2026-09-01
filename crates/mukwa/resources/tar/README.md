# Installation

The bin/ directory contains the binary. The lib/ directory contains the libraries the application needs, these maybe 
already be on your system, but they are included for good measure. The share/ directory contains resources that 
help your desktop environment integrate with the app, such as the app icon and desktop file.

```bash
# Extract the tarball
tar -xzf mukwa-<version>-<arch>.tar.gz

# Install the application
sudo make install
```

The following runtime dependencies are required:

- `zenity`
- `libstdc++`
- `libfreetype`
- `libfontconfig`
- `libm`
- `libc`
- `libz`
- `libbz2`
- `libpng16`
- `libbrotlidec`
- `libexpat`
- `libbrotlicommon`

There is also a make script to install the required dependencies.

```bash
# Arch-based distros
make install-deps-arch
# Debian-based distros 
make install-deps-deb
# Fedora-based distros
make install-deps-fedora
```

