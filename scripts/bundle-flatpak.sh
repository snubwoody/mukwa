#! /bin/sh

flatpak-builder --force-clean --user --repo=repo --install build crates/mukwa/resources/com.wakunguma.Mukwa.yaml
