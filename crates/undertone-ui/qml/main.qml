import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.undertone

QQC2.ApplicationWindow {
    id: window
    visible: true
    width: 800
    height: 600
    minimumWidth: 600
    minimumHeight: 400
    title: "Undertone"

    // Controller from Rust
    UndertoneController {
        id: controller
    }

    // Poll for IPC updates from daemon
    Timer {
        id: updateTimer
        interval: 50  // 20 Hz update rate
        running: true
        repeat: true
        onTriggered: controller.poll_updates()
    }

    // Initialize controller on startup
    Component.onCompleted: {
        controller.initialize()
    }

    // Header bar
    header: QQC2.ToolBar {
        height: 48

        background: Rectangle {
            color: Kirigami.Theme.backgroundColor
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 16
            spacing: 16

            // App title and status
            RowLayout {
                spacing: 8

                QQC2.Label {
                    text: "Undertone"
                    font.pixelSize: 18
                    font.bold: true
                    color: Kirigami.Theme.highlightColor
                }

                // Connection indicator
                Rectangle {
                    width: 8
                    height: 8
                    radius: 4
                    color: controller.connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                }

                QQC2.Label {
                    text: controller.connected ? (controller.device_serial || "Connected") : "Disconnected"
                    font.pixelSize: 12
                    color: Kirigami.Theme.disabledTextColor
                }
            }

            Item { Layout.fillWidth: true }

            // Mix mode toggle
            RowLayout {
                spacing: 4

                QQC2.Button {
                    text: "Stream"
                    flat: true
                    checked: controller.mix_mode === 0
                    checkable: true
                    font.pixelSize: 12
                    onClicked: controller.change_mix_mode(0)

                    background: Rectangle {
                        color: parent.checked ? Kirigami.Theme.highlightColor : "transparent"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: parent.checked ? Kirigami.Theme.highlightedTextColor : Kirigami.Theme.disabledTextColor
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                QQC2.Button {
                    text: "Monitor"
                    flat: true
                    checked: controller.mix_mode === 1
                    checkable: true
                    font.pixelSize: 12
                    onClicked: controller.change_mix_mode(1)

                    background: Rectangle {
                        color: parent.checked ? Kirigami.Theme.highlightColor : "transparent"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: parent.checked ? Kirigami.Theme.highlightedTextColor : Kirigami.Theme.disabledTextColor
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }

            // Profile selector
            RowLayout {
                spacing: 4

                QQC2.ComboBox {
                    id: profileSelector
                    Layout.preferredWidth: 120
                    model: controller.profile_count
                    font.pixelSize: 12

                    property int selectedIdx: 0

                    displayText: controller.profile_name(selectedIdx)

                    onActivated: (index) => {
                        selectedIdx = index
                        controller.load_profile(controller.profile_name(index))
                    }

                    delegate: QQC2.ItemDelegate {
                        required property int index
                        width: profileSelector.width
                        text: controller.profile_name(index) + (controller.profile_is_default(index) ? " *" : "")
                    }
                }

                // Save profile button
                QQC2.Button {
                    Layout.preferredWidth: 32
                    Layout.preferredHeight: 28
                    flat: true
                    text: "+"
                    font.pixelSize: 16
                    font.bold: true

                    onClicked: saveProfileDialog.open()

                    background: Rectangle {
                        color: parent.hovered ? Kirigami.Theme.highlightColor : Kirigami.Theme.alternateBackgroundColor
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Kirigami.Theme.textColor
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }

                    QQC2.ToolTip.visible: hovered
                    QQC2.ToolTip.text: "Save current settings as profile"
                }
            }
        }
    }

    // Tab bar for navigation
    QQC2.TabBar {
        id: tabBar
        width: parent.width
        anchors.top: parent.top

        background: Rectangle {
            color: Kirigami.Theme.alternateBackgroundColor
        }

        QQC2.TabButton {
            text: "Mixer"
            width: implicitWidth
            font.pixelSize: 14

            background: Rectangle {
                color: tabBar.currentIndex === 0 ? Kirigami.Theme.backgroundColor : "transparent"
            }

            contentItem: Text {
                text: parent.text
                color: tabBar.currentIndex === 0 ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }

        QQC2.TabButton {
            text: "Apps"
            width: implicitWidth
            font.pixelSize: 14

            background: Rectangle {
                color: tabBar.currentIndex === 1 ? Kirigami.Theme.backgroundColor : "transparent"
            }

            contentItem: Text {
                text: parent.text
                color: tabBar.currentIndex === 1 ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }

        QQC2.TabButton {
            text: "Device"
            width: implicitWidth
            font.pixelSize: 14

            background: Rectangle {
                color: tabBar.currentIndex === 2 ? Kirigami.Theme.backgroundColor : "transparent"
            }

            contentItem: Text {
                text: parent.text
                color: tabBar.currentIndex === 2 ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }
    }

    // Main content
    StackLayout {
        anchors.top: tabBar.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        currentIndex: tabBar.currentIndex

        // Mixer page
        MixerPage {
            controller: controller
        }

        // Apps page
        AppsPage {
            controller: controller
        }

        // Device page
        DevicePage {
            controller: controller
        }
    }

    // Status bar
    footer: QQC2.ToolBar {
        height: 24

        background: Rectangle {
            color: Kirigami.Theme.backgroundColor
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8

            QQC2.Label {
                text: controller.connected ? "Connected to daemon" : "Daemon not running"
                font.pixelSize: 11
                color: Kirigami.Theme.disabledTextColor
            }

            Item { Layout.fillWidth: true }

            QQC2.Label {
                text: "Profile: " + controller.active_profile
                font.pixelSize: 11
                color: Kirigami.Theme.disabledTextColor
            }
        }
    }

    // Save Profile Dialog
    QQC2.Dialog {
        id: saveProfileDialog
        title: "Save Profile"
        modal: true
        anchors.centerIn: parent
        width: 300

        ColumnLayout {
            anchors.fill: parent
            spacing: 16

            QQC2.TextField {
                id: profileNameField
                Layout.fillWidth: true
                placeholderText: "Enter profile name"
            }
        }

        footer: QQC2.DialogButtonBox {
            QQC2.Button {
                text: "Cancel"
                QQC2.DialogButtonBox.buttonRole: QQC2.DialogButtonBox.RejectRole
            }
            QQC2.Button {
                text: "Save"
                enabled: profileNameField.text.trim().length > 0
                QQC2.DialogButtonBox.buttonRole: QQC2.DialogButtonBox.AcceptRole
            }

            onAccepted: {
                controller.save_profile(profileNameField.text.trim())
                profileNameField.text = ""
                saveProfileDialog.close()
            }
            onRejected: {
                profileNameField.text = ""
                saveProfileDialog.close()
            }
        }
    }
}
