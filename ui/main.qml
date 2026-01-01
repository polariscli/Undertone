import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
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
                    text: controller.connected ? controller.deviceSerial : "Disconnected"
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

            // Profile selector
            ComboBox {
                id: profileSelector
                width: 120
                model: ["Default"]
                currentIndex: 0
                font.pixelSize: 12

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

        // Device page (placeholder)
        Rectangle {
            color: "#1a1a2e"
            Label {
                anchors.centerIn: parent
                text: "Device Settings"
                color: "#94a3b8"
                font.pixelSize: 24
            }
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
}
