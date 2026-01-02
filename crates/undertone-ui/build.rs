//! Build script for undertone-ui.
//!
//! Configures cxx-qt for Qt6/QML integration with Kirigami.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Link KF6 Kirigami via pkg-config
    // This enables native KDE theming support
    if let Err(e) = pkg_config::probe_library("KF6Kirigami") {
        println!("cargo:warning=KF6Kirigami not found via pkg-config: {e}");
        println!("cargo:warning=Install kf6-kirigami-devel (Fedora) or kirigami (Arch)");
    }

    CxxQtBuilder::new()
        .qt_module("Qml")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .qml_module(QmlModule {
            uri: "com.undertone",
            rust_files: &["src/bridge.rs"],
            qml_files: &[
                "qml/main.qml",
                "qml/MixerPage.qml",
                "qml/ChannelStrip.qml",
                "qml/AppsPage.qml",
                "qml/DevicePage.qml",
            ],
            ..Default::default()
        })
        .build();
}
