//! Build script for undertone-ui.
//!
//! Configures cxx-qt for Qt6/QML integration.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
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
