//! Build script for undertone-ui.
//!
//! This will configure cxx-qt when Qt development packages are installed.

fn main() {
    // TODO: Enable cxx-qt build when Qt packages are installed
    //
    // cxx_qt_build::CxxQtBuilder::new()
    //     .file("src/bridge.rs")
    //     .qml_module(cxx_qt_build::QmlModule {
    //         uri: "com.undertone",
    //         rust_files: &["src/bridge.rs"],
    //         qml_files: &["../../ui/main.qml"],
    //         ..Default::default()
    //     })
    //     .build();

    println!("cargo:warning=cxx-qt build disabled - install Qt development packages first");
}
