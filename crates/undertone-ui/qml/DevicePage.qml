import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: devicePage
    color: "#1a1a2e"

    required property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        // Device Status Section
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            color: "#16213e"
            radius: 12

            RowLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 20

                // Device icon
                Rectangle {
                    Layout.preferredWidth: 80
                    Layout.preferredHeight: 80
                    radius: 12
                    color: controller.device_connected ? "#0f3460" : "#2d1f3d"

                    Image {
                        anchors.centerIn: parent
                        width: 48
                        height: 48
                        source: ""
                        visible: false
                    }

                    // Placeholder mic icon
                    Label {
                        anchors.centerIn: parent
                        text: "MIC"
                        font.pixelSize: 16
                        font.bold: true
                        color: controller.device_connected ? "#e94560" : "#64748b"
                    }
                }

                // Device info
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    Label {
                        text: "Elgato Wave:3"
                        font.pixelSize: 20
                        font.bold: true
                        color: "#ffffff"
                    }

                    RowLayout {
                        spacing: 8

                        Rectangle {
                            width: 10
                            height: 10
                            radius: 5
                            color: controller.device_connected ? "#4ade80" : "#ef4444"
                        }

                        Label {
                            text: controller.device_connected ? "Connected" : "Disconnected"
                            font.pixelSize: 14
                            color: controller.device_connected ? "#4ade80" : "#ef4444"
                        }
                    }

                    Label {
                        text: controller.device_connected ? "Serial: " + (controller.device_serial || "Unknown") : "Device not detected"
                        font.pixelSize: 12
                        color: "#64748b"
                    }
                }

                // Connection status badge
                Rectangle {
                    Layout.alignment: Qt.AlignTop
                    width: 100
                    height: 32
                    radius: 16
                    color: controller.device_connected ? "#0f3460" : "#2d1f3d"
                    border.color: controller.device_connected ? "#4ade80" : "#ef4444"
                    border.width: 1

                    Label {
                        anchors.centerIn: parent
                        text: controller.device_connected ? "ONLINE" : "OFFLINE"
                        font.pixelSize: 11
                        font.bold: true
                        color: controller.device_connected ? "#4ade80" : "#ef4444"
                    }
                }
            }
        }

        // Microphone Controls Section
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 200
            color: "#16213e"
            radius: 12
            opacity: controller.device_connected ? 1.0 : 0.5

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 16

                Label {
                    text: "Microphone"
                    font.pixelSize: 16
                    font.bold: true
                    color: "#e94560"
                }

                // Gain slider
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16

                    Label {
                        text: "Gain"
                        font.pixelSize: 14
                        color: "#94a3b8"
                        Layout.preferredWidth: 60
                    }

                    Slider {
                        id: gainSlider
                        Layout.fillWidth: true
                        from: 0
                        to: 1
                        value: controller.mic_gain
                        enabled: controller.device_connected && !controller.mic_muted

                        onMoved: {
                            controller.set_mic_gain_value(value)
                        }

                        background: Rectangle {
                            x: gainSlider.leftPadding
                            y: gainSlider.topPadding + gainSlider.availableHeight / 2 - height / 2
                            width: gainSlider.availableWidth
                            height: 6
                            radius: 3
                            color: "#0f3460"

                            Rectangle {
                                width: gainSlider.visualPosition * parent.width
                                height: parent.height
                                radius: 3
                                color: gainSlider.enabled ? "#e94560" : "#64748b"
                            }
                        }

                        handle: Rectangle {
                            x: gainSlider.leftPadding + gainSlider.visualPosition * (gainSlider.availableWidth - width)
                            y: gainSlider.topPadding + gainSlider.availableHeight / 2 - height / 2
                            width: 20
                            height: 20
                            radius: 10
                            color: gainSlider.pressed ? "#ffffff" : (gainSlider.enabled ? "#e94560" : "#64748b")
                            border.color: "#0f3460"
                            border.width: 2
                        }
                    }

                    Label {
                        text: (isNaN(controller.mic_gain) ? 75 : Math.round(controller.mic_gain * 100)) + "%"
                        font.pixelSize: 14
                        color: "#ffffff"
                        Layout.preferredWidth: 50
                        horizontalAlignment: Text.AlignRight
                    }
                }

                // Mute button
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16

                    Label {
                        text: "Mute"
                        font.pixelSize: 14
                        color: "#94a3b8"
                        Layout.preferredWidth: 60
                    }

                    Button {
                        id: muteButton
                        Layout.preferredWidth: 120
                        Layout.preferredHeight: 40
                        text: controller.mic_muted ? "UNMUTE" : "MUTE"
                        enabled: controller.device_connected

                        onClicked: controller.toggle_mic_mute()

                        background: Rectangle {
                            color: controller.mic_muted ? "#ef4444" : "#0f3460"
                            radius: 8
                            border.color: controller.mic_muted ? "#fca5a5" : "#e94560"
                            border.width: 1
                        }

                        contentItem: Text {
                            text: muteButton.text
                            font.pixelSize: 14
                            font.bold: true
                            color: "#ffffff"
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }

                    Item { Layout.fillWidth: true }

                    // Mute status indicator
                    Rectangle {
                        visible: controller.mic_muted
                        width: 100
                        height: 32
                        radius: 4
                        color: "#3f1f1f"

                        RowLayout {
                            anchors.centerIn: parent
                            spacing: 6

                            Rectangle {
                                width: 8
                                height: 8
                                radius: 4
                                color: "#ef4444"

                                SequentialAnimation on opacity {
                                    running: controller.mic_muted
                                    loops: Animation.Infinite
                                    NumberAnimation { to: 0.3; duration: 500 }
                                    NumberAnimation { to: 1.0; duration: 500 }
                                }
                            }

                            Label {
                                text: "MUTED"
                                font.pixelSize: 11
                                font.bold: true
                                color: "#ef4444"
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }

        // Audio Info Section
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#16213e"
            radius: 12

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 12

                Label {
                    text: "Audio Configuration"
                    font.pixelSize: 16
                    font.bold: true
                    color: "#e94560"
                }

                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 16
                    rowSpacing: 8

                    Label { text: "Sample Rate"; font.pixelSize: 12; color: "#64748b" }
                    Label { text: "48000 Hz"; font.pixelSize: 12; color: "#ffffff" }

                    Label { text: "Bit Depth"; font.pixelSize: 12; color: "#64748b" }
                    Label { text: "24-bit"; font.pixelSize: 12; color: "#ffffff" }

                    Label { text: "Channels"; font.pixelSize: 12; color: "#64748b" }
                    Label { text: "Stereo"; font.pixelSize: 12; color: "#ffffff" }

                    Label { text: "Buffer Size"; font.pixelSize: 12; color: "#64748b" }
                    Label { text: "256 samples"; font.pixelSize: 12; color: "#ffffff" }

                    Label { text: "Latency"; font.pixelSize: 12; color: "#64748b" }
                    Label { text: "~5.3 ms"; font.pixelSize: 12; color: "#ffffff" }
                }

                Item { Layout.fillHeight: true }

                // Footer note
                Label {
                    Layout.fillWidth: true
                    text: "Hardware settings are applied via ALSA fallback. Full HID control is planned for a future release."
                    font.pixelSize: 11
                    color: "#64748b"
                    wrapMode: Text.WordWrap
                }
            }
        }
    }

    // Disconnected overlay
    Rectangle {
        anchors.fill: parent
        color: "#1a1a2e"
        opacity: 0.7
        visible: !controller.device_connected

        MouseArea {
            anchors.fill: parent
            onClicked: {} // Consume clicks
        }

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 16

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "No Device Connected"
                font.pixelSize: 24
                font.bold: true
                color: "#ffffff"
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Please connect your Elgato Wave:3"
                font.pixelSize: 14
                color: "#94a3b8"
            }

            Button {
                Layout.alignment: Qt.AlignHCenter
                text: "Refresh"
                onClicked: controller.refresh()

                background: Rectangle {
                    color: parent.hovered ? "#e94560" : "#0f3460"
                    radius: 8
                    implicitWidth: 120
                    implicitHeight: 40
                }

                contentItem: Text {
                    text: parent.text
                    font.pixelSize: 14
                    color: "#ffffff"
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }
    }
}
