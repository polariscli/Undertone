//! Rust-Qt bridge definitions.
//!
//! This module will contain the cxx-qt bridge definitions that expose
//! Rust types and functions to QML.

// TODO: Implement cxx-qt bridge when Qt development packages are installed
//
// Example structure:
//
// #[cxx_qt::bridge]
// mod qobject {
//     unsafe extern "C++" {
//         include!("cxx-qt-lib/qstring.h");
//         type QString = cxx_qt_lib::QString;
//     }
//
//     unsafe extern "RustQt" {
//         #[qobject]
//         #[qproperty(bool, connected)]
//         #[qproperty(QString, device_serial)]
//         type UndertoneController = super::UndertoneControllerRust;
//     }
// }
