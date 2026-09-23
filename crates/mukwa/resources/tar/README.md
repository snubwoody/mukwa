# Installation

This archive contains compiled binaries for the application. The `bin/` directory contains the app binary. The `share/` 
directory contains resources that help your desktop environment integrate with the app, such as the app icon and 
desktop file.  If you want to know exactly what the `install` command does before running it read 
the [details](#details) below.

```bash
# Extract the tarball
tar -xzf mukwa-<version>-<arch>.tar.gz

# Install the application
cd mukwa-<version>-<arch>.tar.gz
sudo make install
```

## Details

The `install` command installs the `mukwa` binary into your `/usr/local/bin` directory. It also installs the app icon
(mukwa.svg) into the `/usr/share/icons/hicolor/scalable/apps/` directory, the metainfo file into the 
`/usr/share/metainfo/` directory and the desktop file into the `/usr/share/applications/` directory. These help 
your desktop environment detect and run the application.

## Dependencies

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

You may already have these dependencies installed but if you run into errors, there are commands to install the 
dependencies for some popular Linux distros. 

```bash
# Arch-based distros
make install-deps-arch

# Debian-based distros 
make install-deps-deb

# Fedora-based distros
make install-deps-fedora
```

You could as well just install them manually from your terminal.

## Uninstalling

There is also `uninstall` command which deletes all the files installed by the `install` command. 

```bash
sudo make uninstall
```

The `uninstall` command only removes the app but not the application data. If you want to get rid of the app data 
run the `clean` command.

```bash
make clean
```

