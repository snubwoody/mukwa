# Install the application
install:
	install bin/mukwa /usr/local/bin/ -m 0755
	install share/applications/com.wakunguma.Mukwa.desktop /usr/share/applications/com.wakunguma.Mukwa.desktop -m 0644
	install share/metainfo/com.wakunguma.Mukwa.metainfo.xml /usr/share/metainfo/com.wakunguma.Mukwa.metainfo.xml -m 0644
	install share/icons/hicolor/scalable/mukwa.svg /usr/share/icons/hicolor/scalable/apps/mukwa.svg -m 0644

install-deps-arch:
	sudo pacman -Syu zenity xinput

install-deps-deb:
	sudo apt install zenity

install-deps-fedora:
	sudo rpm install zenity

# Uninstall the application
uninstall:
	rm -rf /usr/local/bin/mukwa
	rm -rf /usr/share/applications/com.wakunguma.Mukwa.desktop
	rm -rf /usr/share/metainfo/com.wakunguma.Mukwa.metainfo.xml
	rm -rf /usr/share/icons/hicolor/scalable/apps/mukwa.svg

# Remove the application data
clean:
	rm -rf ~/.local/share/Mukwa/
	rm -rf ~/.local/state/Mukwa/
	rm -rf ~/.config/Mukwa/