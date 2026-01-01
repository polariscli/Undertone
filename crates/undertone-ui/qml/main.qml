import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import com.undertone

ApplicationWindow {
    id: window
    visible: true
    width: 800
    height: 600
    minimumWidth: 600
    minimumHeight: 400
    title: "Undertone"
    color: "#1a1a2e"

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
    header: ToolBar {
        height: 48
        background: Rectangle {
            color: "#16213e"
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 16
            spacing: 16

            // App title and status
            RowLayout {
                spacing: 8

                Label {
                    text: "Undertone"
                    font.pixelSize: 18
                    font.bold: true
                    color: "#e94560"
                }

                // Connection indicator
                Rectangle {
                    width: 8
                    height: 8
                    radius: 4
                    color: controller.connected ? "#4ade80" : "#ef4444"
                }

                Label {
                    text: controller.connected ? (controller.deviceSerial || "Connected") : "Disconnected"
                    font.pixelSize: 12
                    color: "#94a3b8"
                }
            }

            Item { Layout.fillWidth: true }

            // Mix mode toggle
            RowLayout {
                spacing: 4

                Button {
                    text: "Stream"
                    flat: true
                    checked: controller.mixMode === 0
                    checkable: true
                    font.pixelSize: 12
                    onClicked: controller.changeMixMode(0)

                    background: Rectangle {
                        color: parent.checked ? "#e94560" : "transparent"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: parent.checked ? "#ffffff" : "#94a3b8"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                Button {
                    text: "Monitor"
                    flat: true
                    checked: controller.mixMode === 1
                    checkable: true
                    font.pixelSize: 12
                    onClicked: controller.changeMixMode(1)

                    background: Rectangle {
                        color: parent.checked ? "#e94560" : "transparent"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: parent.checked ? "#ffffff" : "#94a3b8"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }

            // Profile selector with save button
            RowLayout {
                spacing: 4

                ComboBox {
                    id: profileSelector
                    Layout.preferredWidth: 120
                    model: controller.profileCount
                    font.pixelSize: 12

                    property int selectedIdx: 0

                    displayText: controller.profileName(selectedIdx)

                    onActivated: (index) => {
                        selectedIdx = index
                        controller.loadProfile(controller.profileName(index))
                    }

                    background: Rectangle {
                        color: "#0f3460"
                        radius: 4
                    }

                    contentItem: Text {
                        text: profileSelector.displayText
                        color: "#ffffff"
                        verticalAlignment: Text.AlignVCenter
                        leftPadding: 8
                    }

                    delegate: ItemDelegate {
                        required property int index
                        width: profileSelector.width
                        height: 28

                        contentItem: RowLayout {
                            Text {
                                text: controller.profileName(index)
                                color: "#ffffff"
                                font.pixelSize: 12
                                Layout.fillWidth: true
                            }
                            Text {
                                text: controller.profileIsDefault(index) ? "*" : ""
                                color: "#e94560"
                                font.pixelSize: 12
                            }
                        }

                        background: Rectangle {
                            color: parent.highlighted ? "#e94560" : "#16213e"
                        }
                    }

                    popup: Popup {
                        y: profileSelector.height
                        width: profileSelector.width
                        implicitHeight: Math.min(contentItem.implicitHeight, 200)
                        padding: 1

                        contentItem: ListView {
                            clip: true
                            implicitHeight: contentHeight
                            model: profileSelector.popup.visible ? profileSelector.delegateModel : null
                            currentIndex: profileSelector.highlightedIndex
                        }

                        background: Rectangle {
                            color: "#16213e"
                            border.color: "#0f3460"
                            radius: 4
                        }
                    }
                }

                // Save profile button
                Button {
                    Layout.preferredWidth: 32
                    Layout.preferredHeight: 28
                    flat: true
                    text: "+"
                    font.pixelSize: 16
                    font.bold: true

                    onClicked: saveProfileDialog.open()

                    background: Rectangle {
                        color: parent.hovered ? "#e94560" : "#0f3460"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: "#ffffff"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }

                    ToolTip.visible: hovered
                    ToolTip.text: "Save current settings as profile"
                }
            }
        }
    }

    // Tab bar for navigation
    TabBar {
        id: tabBar
        width: parent.width
        anchors.top: parent.top

        background: Rectangle {
            color: "#16213e"
        }

        TabButton {
            text: "Mixer"
            width: implicitWidth
            font.pixelSize: 14

            background: Rectangle {
                color: tabBar.currentIndex === 0 ? "#1a1a2e" : "transparent"
            }

            contentItem: Text {
                text: parent.text
                color: tabBar.currentIndex === 0 ? "#e94560" : "#94a3b8"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }

        TabButton {
            text: "Apps"
            width: implicitWidth
            font.pixelSize: 14

            background: Rectangle {
                color: tabBar.currentIndex === 1 ? "#1a1a2e" : "transparent"
            }

            contentItem: Text {
                text: parent.text
                color: tabBar.currentIndex === 1 ? "#e94560" : "#94a3b8"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }

        TabButton {
            text: "Device"
            width: implicitWidth
            font.pixelSize: 14

            background: Rectangle {
                color: tabBar.currentIndex === 2 ? "#1a1a2e" : "transparent"
            }

            contentItem: Text {
                text: parent.text
                color: tabBar.currentIndex === 2 ? "#e94560" : "#94a3b8"
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
    footer: ToolBar {
        height: 24
        background: Rectangle {
            color: "#16213e"
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8

            Label {
                text: controller.connected ? "Connected to daemon" : "Daemon not running"
                font.pixelSize: 11
                color: "#64748b"
            }

            Item { Layout.fillWidth: true }

            Label {
                text: "Profile: " + controller.activeProfile
                font.pixelSize: 11
                color: "#64748b"
            }
        }
    }

    // Save Profile Dialog
    Dialog {
        id: saveProfileDialog
        title: "Save Profile"
        modal: true
        anchors.centerIn: parent
        width: 320
        height: 180

        background: Rectangle {
            color: "#16213e"
            radius: 8
            border.color: "#0f3460"
            border.width: 1
        }

        header: Rectangle {
            height: 40
            color: "#0f3460"
            radius: 8

            Label {
                anchors.centerIn: parent
                text: "Save Profile"
                font.pixelSize: 14
                font.bold: true
                color: "#e94560"
            }

            // Fix bottom corners
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 8
                color: parent.color
            }
        }

        contentItem: ColumnLayout {
            spacing: 16

            Label {
                text: "Enter a name for this profile:"
                font.pixelSize: 12
                color: "#94a3b8"
            }

            TextField {
                id: profileNameField
                Layout.fillWidth: true
                placeholderText: "Profile name"
                font.pixelSize: 14

                background: Rectangle {
                    color: "#0f3460"
                    radius: 4
                    border.color: profileNameField.focus ? "#e94560" : "#16213e"
                    border.width: 1
                }

                color: "#ffffff"
                placeholderTextColor: "#64748b"
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Item { Layout.fillWidth: true }

                Button {
                    text: "Cancel"
                    flat: true
                    onClicked: {
                        profileNameField.text = ""
                        saveProfileDialog.close()
                    }

                    background: Rectangle {
                        color: parent.hovered ? "#0f3460" : "transparent"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: "#94a3b8"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                Button {
                    text: "Save"
                    enabled: profileNameField.text.trim().length > 0

                    onClicked: {
                        controller.saveProfile(profileNameField.text.trim())
                        profileNameField.text = ""
                        saveProfileDialog.close()
                    }

                    background: Rectangle {
                        color: parent.enabled ? (parent.hovered ? "#f05575" : "#e94560") : "#64748b"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: "#ffffff"
                        font.bold: true
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }
}
