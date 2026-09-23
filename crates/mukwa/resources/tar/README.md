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
- `freetype2`
- `fontconfig`
- `libm`
- `glibc`
- `libz`
- `bzip2`
- `libpng`
- `brotli`
- `expat`

You may already have these dependencies installed but install them if you run into errors.

## Uninstalling

There is an `uninstall` command which deletes all the files installed by the `install` command. 

```bash
sudo make uninstall
```

The `uninstall` command only removes the app and not the application data. If you want to get rid of the app data 
run the `clean` command.

```bash
make clean
```

