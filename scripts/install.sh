#!/bin/bash
# Undertone Installation Script
# https://github.com/polariscli/Undertone
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/polariscli/Undertone/main/scripts/install.sh | bash
#   curl -sSL https://raw.githubusercontent.com/polariscli/Undertone/main/scripts/install.sh | bash -s -- uninstall
#
# Or run locally:
#   ./scripts/install.sh [command]

set -e

# Configuration
REPO_URL="https://github.com/polariscli/Undertone.git"
INSTALL_DIR="${UNDERTONE_INSTALL_DIR:-$HOME/.local/share/undertone-src}"
BIN_DIR="${UNDERTONE_BIN_DIR:-$HOME/.cargo/bin}"

# Colors (disabled if not a terminal)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    BOLD=''
    NC=''
fi

# Paths
UDEV_RULES="/etc/udev/rules.d/99-elgato-wave3.rules"
WP_CONF_DIR="$HOME/.config/wireplumber/wireplumber.conf.d"
WP_SCRIPT_DIR="$HOME/.config/wireplumber/scripts"
SYSTEMD_DIR="$HOME/.config/systemd/user"
DATA_DIR="$HOME/.local/share/undertone"

# Print functions
print_header() {
    echo -e "${BOLD}${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

print_info() {
    echo -e "${CYAN}→${NC} $1"
}

# Detect the source directory (git repo or installed location)
detect_source_dir() {
    # Check if we're running from within the git repo
    local script_path="${BASH_SOURCE[0]}"

    # If running via curl/pipe, script_path might be empty or /dev/stdin
    if [[ -z "$script_path" || "$script_path" == "/dev/stdin" || "$script_path" == "-" ]]; then
        # Running via pipe, need to clone repo
        return 1
    fi

    # Resolve to absolute path
    script_path="$(cd "$(dirname "$script_path")" 2>/dev/null && pwd)/$(basename "$script_path")" 2>/dev/null || return 1

    # Check if we're in a git repo with undertone
    local potential_project_dir="$(dirname "$(dirname "$script_path")")"
    if [[ -f "$potential_project_dir/Cargo.toml" ]] && grep -q 'name = "undertone"' "$potential_project_dir/Cargo.toml" 2>/dev/null; then
        PROJECT_DIR="$potential_project_dir"
        return 0
    fi

    # Check if already cloned to install dir
    if [[ -f "$INSTALL_DIR/Cargo.toml" ]] && grep -q 'name = "undertone"' "$INSTALL_DIR/Cargo.toml" 2>/dev/null; then
        PROJECT_DIR="$INSTALL_DIR"
        return 0
    fi

    return 1
}

# Clone or update the repository
ensure_repo() {
    if detect_source_dir; then
        print_info "Using source from: $PROJECT_DIR"
        return 0
    fi

    print_info "Cloning Undertone repository..."

    if [[ -d "$INSTALL_DIR" ]]; then
        print_warning "Removing existing installation directory..."
        rm -rf "$INSTALL_DIR"
    fi

    if ! command -v git &>/dev/null; then
        print_error "git is required but not installed"
        exit 1
    fi

    git clone --depth 1 "$REPO_URL" "$INSTALL_DIR"
    PROJECT_DIR="$INSTALL_DIR"
    print_success "Cloned to $PROJECT_DIR"
}

# Update the repository
update_repo() {
    if ! detect_source_dir; then
        print_error "Undertone is not installed. Run 'install' first."
        exit 1
    fi

    print_info "Updating Undertone repository..."
    cd "$PROJECT_DIR"

    if [[ -d ".git" ]]; then
        git fetch origin
        git reset --hard origin/main
        print_success "Updated to latest version"
    else
        print_warning "Not a git repository, cannot update. Reinstalling..."
        rm -rf "$INSTALL_DIR"
        ensure_repo
    fi
}

# Detect package manager and OS
detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS_ID="$ID"
        OS_NAME="$NAME"
    elif command -v lsb_release &>/dev/null; then
        OS_ID="$(lsb_release -si | tr '[:upper:]' '[:lower:]')"
        OS_NAME="$(lsb_release -sd)"
    else
        OS_ID="unknown"
        OS_NAME="Unknown Linux"
    fi
}

# Check for required dependencies
check_dependencies() {
    print_header "Checking dependencies..."
    local missing=()
    local install_cmd=""

    detect_os

    # Check for Rust/Cargo
    if ! command -v cargo &>/dev/null; then
        print_error "Rust/Cargo is not installed"
        echo ""
        echo "Install Rust with:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        exit 1
    fi
    print_success "Rust/Cargo found"

    # Check cargo version for Edition 2024 support (Rust 1.85+)
    local rust_version=$(rustc --version | grep -oP '\d+\.\d+' | head -1)
    local rust_major=$(echo "$rust_version" | cut -d. -f1)
    local rust_minor=$(echo "$rust_version" | cut -d. -f2)
    if [[ "$rust_major" -lt 1 || ("$rust_major" -eq 1 && "$rust_minor" -lt 85) ]]; then
        print_warning "Rust $rust_version detected. Rust 1.85+ recommended for Edition 2024"
        echo "  Update with: rustup update"
    else
        print_success "Rust version $rust_version (Edition 2024 supported)"
    fi

    # Check for pkg-config
    if ! command -v pkg-config &>/dev/null; then
        missing+=("pkg-config")
    else
        print_success "pkg-config found"
    fi

    # Check for clang (required by cxx-qt)
    if ! command -v clang &>/dev/null; then
        missing+=("clang")
    else
        print_success "clang found"
    fi

    # Check for PipeWire development files
    if ! pkg-config --exists libpipewire-0.3 2>/dev/null; then
        case "$OS_ID" in
            fedora|rhel|centos)
                missing+=("pipewire-devel")
                ;;
            arch|manjaro|endeavouros)
                missing+=("pipewire")
                ;;
            ubuntu|debian|linuxmint|pop)
                missing+=("libpipewire-0.3-dev")
                ;;
            *)
                missing+=("pipewire-dev (or equivalent)")
                ;;
        esac
    else
        print_success "PipeWire development files found"
    fi

    # Check for Qt6
    if ! pkg-config --exists Qt6Core 2>/dev/null; then
        case "$OS_ID" in
            fedora|rhel|centos)
                missing+=("qt6-qtbase-devel" "qt6-qtdeclarative-devel")
                ;;
            arch|manjaro|endeavouros)
                missing+=("qt6-base" "qt6-declarative")
                ;;
            ubuntu|debian|linuxmint|pop)
                missing+=("qt6-base-dev" "qt6-declarative-dev")
                ;;
            *)
                missing+=("qt6-dev (or equivalent)")
                ;;
        esac
    else
        print_success "Qt6 development files found"
    fi

    # Check for Kirigami
    if ! pkg-config --exists KF6Kirigami 2>/dev/null; then
        case "$OS_ID" in
            fedora|rhel|centos)
                missing+=("kf6-kirigami-devel" "kf6-qqc2-desktop-style")
                ;;
            arch|manjaro|endeavouros)
                missing+=("kirigami")
                ;;
            ubuntu|debian|linuxmint|pop)
                missing+=("libkf6kirigami-dev")
                ;;
            *)
                missing+=("kirigami6-dev (or equivalent)")
                ;;
        esac
    else
        print_success "Kirigami6 development files found"
    fi

    # Report missing dependencies
    if [[ ${#missing[@]} -gt 0 ]]; then
        echo ""
        print_error "Missing dependencies: ${missing[*]}"
        echo ""

        case "$OS_ID" in
            fedora|rhel|centos)
                install_cmd="sudo dnf install ${missing[*]}"
                ;;
            arch|manjaro|endeavouros)
                install_cmd="sudo pacman -S ${missing[*]}"
                ;;
            ubuntu|debian|linuxmint|pop)
                install_cmd="sudo apt install ${missing[*]}"
                ;;
        esac

        if [[ -n "$install_cmd" ]]; then
            echo "Install with:"
            echo "  $install_cmd"
            echo ""
        fi

        exit 1
    fi

    echo ""
    print_success "All dependencies satisfied"
    echo ""
}

# Install udev rules for Wave:3 access
install_udev() {
    print_info "Installing udev rules for Wave:3 access..."

    if [[ ! -f "$PROJECT_DIR/scripts/99-elgato-wave3.rules" ]]; then
        print_error "udev rules file not found"
        return 1
    fi

    if [[ "$EUID" -eq 0 ]]; then
        cp "$PROJECT_DIR/scripts/99-elgato-wave3.rules" "$UDEV_RULES"
        udevadm control --reload-rules
        udevadm trigger
    else
        sudo cp "$PROJECT_DIR/scripts/99-elgato-wave3.rules" "$UDEV_RULES"
        sudo udevadm control --reload-rules
        sudo udevadm trigger
    fi

    print_success "udev rules installed"
}

# Uninstall udev rules
uninstall_udev() {
    print_info "Removing udev rules..."

    if [[ -f "$UDEV_RULES" ]]; then
        if [[ "$EUID" -eq 0 ]]; then
            rm -f "$UDEV_RULES"
            udevadm control --reload-rules
        else
            sudo rm -f "$UDEV_RULES"
            sudo udevadm control --reload-rules
        fi
        print_success "udev rules removed"
    else
        print_info "udev rules not found (already removed)"
    fi
}

# Install WirePlumber configuration
install_wireplumber() {
    print_info "Installing WirePlumber configuration..."

    mkdir -p "$WP_CONF_DIR"
    mkdir -p "$WP_SCRIPT_DIR"

    cp "$PROJECT_DIR/scripts/wireplumber/51-elgato.conf" "$WP_CONF_DIR/"
    cp "$PROJECT_DIR/scripts/wireplumber/elgato_wave_3.lua" "$WP_SCRIPT_DIR/"

    print_success "WirePlumber configuration installed"
    print_warning "Restart WirePlumber to apply: systemctl --user restart wireplumber"
}

# Uninstall WirePlumber configuration
uninstall_wireplumber() {
    print_info "Removing WirePlumber configuration..."

    rm -f "$WP_CONF_DIR/51-elgato.conf"
    rm -f "$WP_SCRIPT_DIR/elgato_wave_3.lua"

    # Remove empty directories
    rmdir "$WP_SCRIPT_DIR" 2>/dev/null || true
    rmdir "$WP_CONF_DIR" 2>/dev/null || true
    rmdir "$(dirname "$WP_CONF_DIR")" 2>/dev/null || true

    print_success "WirePlumber configuration removed"
}

# Build and install binaries
build_and_install() {
    print_info "Building Undertone (release mode)..."

    cd "$PROJECT_DIR"
    cargo build --release -p undertone-daemon -p undertone-ui

    print_info "Installing binaries to $BIN_DIR..."
    mkdir -p "$BIN_DIR"

    cp target/release/undertone-daemon "$BIN_DIR/"
    cp target/release/undertone "$BIN_DIR/"
    chmod +x "$BIN_DIR/undertone-daemon"
    chmod +x "$BIN_DIR/undertone"

    print_success "Binaries installed: undertone-daemon, undertone"

    # Check if BIN_DIR is in PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        print_warning "$BIN_DIR is not in your PATH"
        echo "  Add to your shell config:"
        echo "    export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    fi
}

# Uninstall binaries
uninstall_binaries() {
    print_info "Removing binaries..."

    rm -f "$BIN_DIR/undertone-daemon"
    rm -f "$BIN_DIR/undertone"

    print_success "Binaries removed"
}

# Install systemd user service
install_service() {
    print_info "Installing systemd user service..."

    mkdir -p "$SYSTEMD_DIR"
    cp "$PROJECT_DIR/scripts/undertone-daemon.service" "$SYSTEMD_DIR/"
    systemctl --user daemon-reload

    print_success "Systemd service installed"
}

# Uninstall systemd service
uninstall_service() {
    print_info "Stopping and removing systemd service..."

    # Stop and disable if running
    systemctl --user stop undertone-daemon 2>/dev/null || true
    systemctl --user disable undertone-daemon 2>/dev/null || true

    rm -f "$SYSTEMD_DIR/undertone-daemon.service"
    systemctl --user daemon-reload

    print_success "Systemd service removed"
}

# Remove application data
remove_data() {
    print_info "Removing application data..."

    if [[ -d "$DATA_DIR" ]]; then
        rm -rf "$DATA_DIR"
        print_success "Application data removed"
    else
        print_info "No application data found"
    fi
}

# Remove source directory
remove_source() {
    print_info "Removing source directory..."

    if [[ -d "$INSTALL_DIR" ]]; then
        rm -rf "$INSTALL_DIR"
        print_success "Source directory removed"
    else
        print_info "Source directory not found (may be using git clone location)"
    fi
}

# Full installation
cmd_install() {
    echo ""
    print_header "╔══════════════════════════════════════╗"
    print_header "║     Undertone Installation Script    ║"
    print_header "╚══════════════════════════════════════╝"
    echo ""

    check_dependencies
    ensure_repo

    echo ""
    print_header "Installing components..."
    echo ""

    install_udev
    install_wireplumber
    build_and_install
    install_service

    echo ""
    print_header "════════════════════════════════════════"
    print_success "Installation complete!"
    print_header "════════════════════════════════════════"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Restart WirePlumber (for Wave:3 support):"
    echo "     ${CYAN}systemctl --user restart wireplumber${NC}"
    echo ""
    echo "  2. Start the daemon:"
    echo "     ${CYAN}systemctl --user start undertone-daemon${NC}"
    echo ""
    echo "  3. Enable on login (optional):"
    echo "     ${CYAN}systemctl --user enable undertone-daemon${NC}"
    echo ""
    echo "  4. Run the UI:"
    echo "     ${CYAN}undertone${NC}"
    echo ""
}

# Full uninstallation
cmd_uninstall() {
    echo ""
    print_header "╔══════════════════════════════════════╗"
    print_header "║    Undertone Uninstallation Script   ║"
    print_header "╚══════════════════════════════════════╝"
    echo ""

    uninstall_service
    uninstall_binaries
    uninstall_wireplumber
    uninstall_udev

    echo ""
    read -p "Remove application data (~/.local/share/undertone)? [y/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        remove_data
    fi

    read -p "Remove source directory ($INSTALL_DIR)? [y/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        remove_source
    fi

    echo ""
    print_header "════════════════════════════════════════"
    print_success "Uninstallation complete!"
    print_header "════════════════════════════════════════"
    echo ""
    echo "Note: Restart WirePlumber to restore default audio config:"
    echo "  ${CYAN}systemctl --user restart wireplumber${NC}"
    echo ""
}

# Update installation
cmd_update() {
    echo ""
    print_header "╔══════════════════════════════════════╗"
    print_header "║      Undertone Update Script         ║"
    print_header "╚══════════════════════════════════════╝"
    echo ""

    update_repo
    check_dependencies

    # Stop service if running
    if systemctl --user is-active undertone-daemon &>/dev/null; then
        print_info "Stopping undertone-daemon..."
        systemctl --user stop undertone-daemon
        local was_running=true
    fi

    build_and_install
    install_service  # In case service file changed
    install_wireplumber  # In case config changed

    if [[ "$was_running" == "true" ]]; then
        print_info "Restarting undertone-daemon..."
        systemctl --user start undertone-daemon
    fi

    echo ""
    print_header "════════════════════════════════════════"
    print_success "Update complete!"
    print_header "════════════════════════════════════════"
    echo ""
}

# Service management
cmd_start() {
    systemctl --user start undertone-daemon
    print_success "undertone-daemon started"
}

cmd_stop() {
    systemctl --user stop undertone-daemon
    print_success "undertone-daemon stopped"
}

cmd_enable() {
    systemctl --user enable undertone-daemon
    print_success "undertone-daemon enabled on login"
}

cmd_disable() {
    systemctl --user disable undertone-daemon
    print_success "undertone-daemon disabled on login"
}

cmd_status() {
    systemctl --user status undertone-daemon
}

cmd_logs() {
    journalctl --user -u undertone-daemon -f
}

# Show help
show_help() {
    echo ""
    print_header "Undertone Installation Script"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  ${BOLD}install${NC}       Full installation (default)"
    echo "  ${BOLD}uninstall${NC}     Remove Undertone completely"
    echo "  ${BOLD}update${NC}        Update to latest version"
    echo ""
    echo "  ${BOLD}udev${NC}          Install only udev rules"
    echo "  ${BOLD}wireplumber${NC}   Install only WirePlumber config"
    echo "  ${BOLD}service${NC}       Install only systemd service"
    echo "  ${BOLD}build${NC}         Build and install binaries only"
    echo ""
    echo "  ${BOLD}start${NC}         Start the daemon"
    echo "  ${BOLD}stop${NC}          Stop the daemon"
    echo "  ${BOLD}enable${NC}        Enable daemon on login"
    echo "  ${BOLD}disable${NC}       Disable daemon on login"
    echo "  ${BOLD}status${NC}        Show daemon status"
    echo "  ${BOLD}logs${NC}          Follow daemon logs"
    echo ""
    echo "  ${BOLD}check${NC}         Check dependencies only"
    echo "  ${BOLD}help${NC}          Show this help"
    echo ""
    echo "Quick install:"
    echo "  curl -sSL https://raw.githubusercontent.com/polariscli/Undertone/main/scripts/install.sh | bash"
    echo ""
    echo "Environment variables:"
    echo "  UNDERTONE_INSTALL_DIR  Source directory (default: ~/.local/share/undertone-src)"
    echo "  UNDERTONE_BIN_DIR      Binary directory (default: ~/.cargo/bin)"
    echo ""
}

# Main entry point
main() {
    case "${1:-install}" in
        install)
            cmd_install
            ;;
        uninstall|remove)
            cmd_uninstall
            ;;
        update|upgrade)
            cmd_update
            ;;
        udev)
            ensure_repo
            install_udev
            ;;
        wireplumber|wp)
            ensure_repo
            install_wireplumber
            ;;
        service|systemd)
            ensure_repo
            install_service
            ;;
        build)
            check_dependencies
            ensure_repo
            build_and_install
            ;;
        start)
            cmd_start
            ;;
        stop)
            cmd_stop
            ;;
        enable)
            cmd_enable
            ;;
        disable)
            cmd_disable
            ;;
        status)
            cmd_status
            ;;
        logs|log)
            cmd_logs
            ;;
        check|deps)
            check_dependencies
            ;;
        help|-h|--help)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
