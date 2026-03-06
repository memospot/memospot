#!/usr/bin/env bash
set -e

UBUNTU_CODENAME="${1:-jammy}"

case "$UBUNTU_CODENAME" in
jammy | noble) ;;
*)
    echo "Unsupported Ubuntu codename: $UBUNTU_CODENAME"
    echo "Supported values: jammy, noble"
    exit 1
    ;;
esac

if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (or with sudo)"
    exit 1
fi

echo "Configuring Ubuntu Snapshot repository for WebKit2GTK 2.48 on Ubuntu (${UBUNTU_CODENAME})..."
cat <<EOF >/etc/apt/sources.list.d/webkit2gtk-snapshot.list
deb [check-valid-until=no] https://snapshot.ubuntu.com/ubuntu/20251101T000000Z ${UBUNTU_CODENAME}-updates main universe
deb [check-valid-until=no] https://snapshot.ubuntu.com/ubuntu/20251101T000000Z ${UBUNTU_CODENAME}-security main universe
EOF

echo "Setting APT preferences to pin WebKit2GTK versions to 2.48.*..."
cat <<"EOF" >/etc/apt/preferences.d/webkit2gtk-pin
Package: *webkit2gtk* *javascriptcoregtk* gir1.2-javascriptcoregtk-4.1 gir1.2-webkit2-4.1
Pin: version 2.48.*
Pin-Priority: 1001
EOF

echo "Updating APT cache..."
apt-get -o Acquire::Check-Valid-Until=false update -q

echo "Installing WebKit2GTK 2.48 packages..."
apt-get install -y --allow-downgrades \
    libwebkit2gtk-4.1-0 \
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-0 \
    libjavascriptcoregtk-4.1-dev \
    gir1.2-webkit2-4.1 \
    gir1.2-javascriptcoregtk-4.1

echo "Installation complete. Checking installed versions:"
dpkg -l | grep -E "webkit2gtk|javascriptcoregtk"
