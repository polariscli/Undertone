//! Build script for undertone-pipewire.
//!
//! Checks that libpipewire is available.

fn main() {
    // Check for libpipewire
    if let Err(e) = pkg_config::probe_library("libpipewire-0.3") {
        eprintln!("Warning: libpipewire-0.3 not found: {e}");
        eprintln!("Install pipewire-devel (Fedora) or libpipewire-0.3-dev (Debian/Ubuntu)");
        // Don't fail the build - the crate can still compile, just won't link
    }
}
