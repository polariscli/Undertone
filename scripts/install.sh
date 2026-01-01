#!/bin/bash
# Undertone installation script
# Installs WirePlumber config, systemd service, and udev rules for the Elgato Wave:3

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Undertone Installation Script"
echo "=============================="
echo ""

# Install WirePlumber configuration for Wave:3
install_wireplumber() {
    echo "Installing WirePlumber configuration for Wave:3..."

    WPCONF_DIR="$HOME/.config/wireplumber/wireplumber.conf.d"
    WPSCRIPT_DIR="$HOME/.config/wireplumber/scripts"

    mkdir -p "$WPCONF_DIR"
    mkdir -p "$WPSCRIPT_DIR"

    # Copy configuration
    cp "$SCRIPT_DIR/wireplumber/51-elgato.conf" "$WPCONF_DIR/"
    cp "$SCRIPT_DIR/wireplumber/elgato_wave_3.lua" "$WPSCRIPT_DIR/"

    echo "  Installed WirePlumber config to $WPCONF_DIR"
    echo "  Installed WirePlumber script to $WPSCRIPT_DIR"
    echo "  Done!"
    echo ""
    echo "  NOTE: Restart WirePlumber to apply changes:"
    echo "    systemctl --user restart wireplumber"
}

# Check if running as root (for udev rules)
install_udev_rules() {
    if [ "$EUID" -eq 0 ]; then
        echo "Installing udev rules..."
        cp "$SCRIPT_DIR/99-elgato-wave3.rules" /etc/udev/rules.d/
        udevadm control --reload-rules
        udevadm trigger
        echo "  Done!"
    else
        echo "Installing udev rules (requires sudo)..."
        sudo cp "$SCRIPT_DIR/99-elgato-wave3.rules" /etc/udev/rules.d/
        sudo udevadm control --reload-rules
        sudo udevadm trigger
        echo "  Done!"
    fi
}

# Install systemd user service
install_service() {
    echo "Installing systemd user service..."
    mkdir -p "$HOME/.config/systemd/user"
    cp "$SCRIPT_DIR/undertone-daemon.service" "$HOME/.config/systemd/user/"
    systemctl --user daemon-reload
    echo "  Done!"
}

# Build and install binaries
build_and_install() {
    echo "Building Undertone..."
    cd "$PROJECT_DIR"
    cargo build --release -p undertone-daemon -p undertone-ui

    echo "Installing binaries to ~/.cargo/bin..."
    mkdir -p "$HOME/.cargo/bin"
    cp target/release/undertone-daemon "$HOME/.cargo/bin/"
    cp target/release/undertone "$HOME/.cargo/bin/"
    echo "  Done!"
}

# Main installation
main() {
    echo "1. Installing udev rules for Wave:3 access..."
    install_udev_rules
    echo ""

    echo "2. Installing WirePlumber configuration..."
    install_wireplumber
    echo ""

    echo "3. Building and installing binaries..."
    build_and_install
    echo ""

    echo "4. Installing systemd service..."
    install_service
    echo ""

    echo "Installation complete!"
    echo ""
    echo "IMPORTANT: Restart WirePlumber to activate Wave:3 support:"
    echo "  systemctl --user restart wireplumber"
    echo ""
    echo "To start the daemon:"
    echo "  systemctl --user start undertone-daemon"
    echo ""
    echo "To enable on login:"
    echo "  systemctl --user enable undertone-daemon"
    echo ""
    echo "To run the UI:"
    echo "  undertone"
}

# Parse arguments
case "${1:-install}" in
    install)
        main
        ;;
    udev)
        install_udev_rules
        ;;
    wireplumber)
        install_wireplumber
        ;;
    service)
        install_service
        ;;
    build)
        build_and_install
        ;;
    *)
        echo "Usage: $0 [install|udev|wireplumber|service|build]"
        echo ""
        echo "Commands:"
        echo "  install     - Full installation (default)"
        echo "  udev        - Install only udev rules"
        echo "  wireplumber - Install only WirePlumber config for Wave:3"
        echo "  service     - Install only systemd service"
        echo "  build       - Build and install binaries only"
        exit 1
        ;;
esac
