import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: devicePage
    color: Kirigami.Theme.backgroundColor

    required property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        // Device Status Section
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            color: Kirigami.Theme.alternateBackgroundColor
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
                    color: controller.device_connected ? Kirigami.Theme.backgroundColor : Kirigami.Theme.alternateBackgroundColor

                    // Placeholder mic icon
                    QQC2.Label {
                        anchors.centerIn: parent
                        text: "MIC"
                        font.pixelSize: 16
                        font.bold: true
                        color: controller.device_connected ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor
                    }
                }

                // Device info
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    QQC2.Label {
                        text: "Elgato Wave:3"
                        font.pixelSize: 20
                        font.bold: true
                        color: Kirigami.Theme.textColor
                    }

                    RowLayout {
                        spacing: 8

                        Rectangle {
                            width: 10
                            height: 10
                            radius: 5
                            color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                        }

                        QQC2.Label {
                            text: controller.device_connected ? "Connected" : "Disconnected"
                            font.pixelSize: 14
                            color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                        }
                    }

                    QQC2.Label {
                        text: controller.device_connected ? "Serial: " + (controller.device_serial || "Unknown") : "Device not detected"
                        font.pixelSize: 12
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                // Connection status badge
                Rectangle {
                    Layout.alignment: Qt.AlignTop
                    width: 100
                    height: 32
                    radius: 16
                    color: Kirigami.Theme.backgroundColor
                    border.color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                    border.width: 1

                    QQC2.Label {
                        anchors.centerIn: parent
                        text: controller.device_connected ? "ONLINE" : "OFFLINE"
                        font.pixelSize: 11
                        font.bold: true
                        color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                    }
                }
            }
        }

        // Microphone Controls Section
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 200
            color: Kirigami.Theme.alternateBackgroundColor
            radius: 12
            opacity: controller.device_connected ? 1.0 : 0.5

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 16

                QQC2.Label {
                    text: "Microphone"
                    font.pixelSize: 16
                    font.bold: true
                    color: Kirigami.Theme.highlightColor
                }

                // Gain slider
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16

                    QQC2.Label {
                        text: "Gain"
                        font.pixelSize: 14
                        color: Kirigami.Theme.disabledTextColor
                        Layout.preferredWidth: 60
                    }

                    QQC2.Slider {
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
                            color: Kirigami.Theme.backgroundColor

                            Rectangle {
                                width: gainSlider.visualPosition * parent.width
                                height: parent.height
                                radius: 3
                                color: gainSlider.enabled ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor
                            }
                        }

                        handle: Rectangle {
                            x: gainSlider.leftPadding + gainSlider.visualPosition * (gainSlider.availableWidth - width)
                            y: gainSlider.topPadding + gainSlider.availableHeight / 2 - height / 2
                            width: 20
                            height: 20
                            radius: 10
                            color: gainSlider.pressed ? Kirigami.Theme.textColor : (gainSlider.enabled ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor)
                            border.color: Kirigami.Theme.backgroundColor
                            border.width: 2
                        }
                    }

                    QQC2.Label {
                        text: (isNaN(controller.mic_gain) ? 75 : Math.round(controller.mic_gain * 100)) + "%"
                        font.pixelSize: 14
                        color: Kirigami.Theme.textColor
                        Layout.preferredWidth: 50
                        horizontalAlignment: Text.AlignRight
                    }
                }

                // Mute button
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16

                    QQC2.Label {
                        text: "Mute"
                        font.pixelSize: 14
                        color: Kirigami.Theme.disabledTextColor
                        Layout.preferredWidth: 60
                    }

                    QQC2.Button {
                        id: muteButton
                        Layout.preferredWidth: 120
                        Layout.preferredHeight: 40
                        text: controller.mic_muted ? "UNMUTE" : "MUTE"
                        enabled: controller.device_connected

                        onClicked: controller.toggle_mic_mute()

                        background: Rectangle {
                            color: controller.mic_muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.backgroundColor
                            radius: 8
                            border.color: controller.mic_muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.highlightColor
                            border.width: 1
                        }

                        contentItem: Text {
                            text: muteButton.text
                            font.pixelSize: 14
                            font.bold: true
                            color: Kirigami.Theme.textColor
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
                        color: Qt.rgba(Kirigami.Theme.negativeTextColor.r, Kirigami.Theme.negativeTextColor.g, Kirigami.Theme.negativeTextColor.b, 0.2)

                        RowLayout {
                            anchors.centerIn: parent
                            spacing: 6

                            Rectangle {
                                width: 8
                                height: 8
                                radius: 4
                                color: Kirigami.Theme.negativeTextColor

                                SequentialAnimation on opacity {
                                    running: controller.mic_muted
                                    loops: Animation.Infinite
                                    NumberAnimation { to: 0.3; duration: 500 }
                                    NumberAnimation { to: 1.0; duration: 500 }
                                }
                            }

                            QQC2.Label {
                                text: "MUTED"
                                font.pixelSize: 11
                                font.bold: true
                                color: Kirigami.Theme.negativeTextColor
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
            color: Kirigami.Theme.alternateBackgroundColor
            radius: 12

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 12

                QQC2.Label {
                    text: "Audio Configuration"
                    font.pixelSize: 16
                    font.bold: true
                    color: Kirigami.Theme.highlightColor
                }

                Kirigami.FormLayout {
                    Layout.fillWidth: true

                    QQC2.Label {
                        Kirigami.FormData.label: "Sample Rate:"
                        text: "48000 Hz (USB Audio)"
                    }

                    QQC2.Label {
                        Kirigami.FormData.label: "Bit Depth:"
                        text: "24-bit"
                    }

                    QQC2.Label {
                        Kirigami.FormData.label: "Audio Channels:"
                        text: "Stereo (2ch)"
                    }

                    QQC2.Label {
                        Kirigami.FormData.label: "Buffer/Latency:"
                        text: "Managed by PipeWire"
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                Item { Layout.fillHeight: true }

                // Footer note
                QQC2.Label {
                    Layout.fillWidth: true
                    text: "Audio routing managed by PipeWire. Mic gain controlled via ALSA. Buffer size configurable in PipeWire/WirePlumber settings."
                    font.pixelSize: 11
                    color: Kirigami.Theme.disabledTextColor
                    wrapMode: Text.WordWrap
                }
            }
        }
    }

    // Disconnected overlay
    Rectangle {
        anchors.fill: parent
        color: Kirigami.Theme.backgroundColor
        opacity: 0.85
        visible: !controller.device_connected

        MouseArea {
            anchors.fill: parent
            onClicked: {} // Consume clicks
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            icon.name: "audio-headphones"
            text: "No Device Connected"
            explanation: "Please connect your Elgato Wave:3"

            helpfulAction: Kirigami.Action {
                text: "Refresh"
                icon.name: "view-refresh"
                onTriggered: controller.refresh()
            }
        }
    }
}
