import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: devicePage
    color: Kirigami.Theme.backgroundColor

    required property var controller

    // Track gain slider value locally to prevent binding issues during drag
    property real localGainValue: controller.mic_gain

    Connections {
        target: controller
        function onMic_gainChanged() {
            if (!gainSlider.pressed) {
                localGainValue = controller.mic_gain
            }
        }
    }

    Flickable {
        id: flickable
        anchors.fill: parent
        contentWidth: width
        contentHeight: contentColumn.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        // Only scroll via mouse wheel, don't capture drag
        interactive: false

        // Enable mouse wheel scrolling
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.NoButton
            onWheel: (wheel) => {
                flickable.contentY = Math.max(0, Math.min(
                    flickable.contentHeight - flickable.height,
                    flickable.contentY - wheel.angleDelta.y
                ))
            }
        }

        ColumnLayout {
            id: contentColumn
            width: flickable.width
            spacing: 16

            // Top margin
            Item { Layout.preferredHeight: 24 }

            // Device Status Section
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                implicitHeight: deviceStatusContent.implicitHeight + 40
                color: Kirigami.Theme.alternateBackgroundColor
                radius: 12

                RowLayout {
                    id: deviceStatusContent
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.margins: 20
                    spacing: 12

                    // Device icon - the good version
                    Rectangle {
                        Layout.preferredWidth: 64
                        Layout.preferredHeight: 64
                        radius: 10
                        color: controller.device_connected ? Kirigami.Theme.backgroundColor : Kirigami.Theme.alternateBackgroundColor

                        Kirigami.Icon {
                            anchors.centerIn: parent
                            width: 32
                            height: 32
                            source: "audio-input-microphone"
                            color: controller.device_connected ? Kirigami.Theme.highlightColor : Kirigami.Theme.disabledTextColor
                        }
                    }

                    // Device info
                    ColumnLayout {
                        spacing: 2

                        RowLayout {
                            spacing: 12

                            QQC2.Label {
                                text: "Elgato Wave:3"
                                font.pixelSize: 18
                                font.bold: true
                                color: Kirigami.Theme.textColor
                            }

                            Rectangle {
                                width: 70
                                height: 22
                                radius: 11
                                color: "transparent"
                                border.color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                                border.width: 1

                                QQC2.Label {
                                    anchors.centerIn: parent
                                    text: controller.device_connected ? "ONLINE" : "OFFLINE"
                                    font.pixelSize: 9
                                    font.bold: true
                                    color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                                }
                            }
                        }

                        RowLayout {
                            spacing: 6

                            Rectangle {
                                width: 8
                                height: 8
                                radius: 4
                                color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                            }

                            QQC2.Label {
                                text: controller.device_connected ? "Connected" : "Disconnected"
                                font.pixelSize: 12
                                color: controller.device_connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                            }
                        }

                        QQC2.Label {
                            visible: controller.device_connected
                            text: "Serial: " + (controller.device_serial || "Unknown")
                            font.pixelSize: 11
                            color: Kirigami.Theme.disabledTextColor
                        }
                    }
                }
            }

            // Microphone Controls Section
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                implicitHeight: micColumn.implicitHeight + 40
                color: Kirigami.Theme.alternateBackgroundColor
                radius: 12
                opacity: controller.device_connected ? 1.0 : 0.5

                ColumnLayout {
                    id: micColumn
                    anchors.fill: parent
                    anchors.margins: 20
                    spacing: 16

                    RowLayout {
                        spacing: 8

                        QQC2.Label {
                            text: "Microphone"
                            font.pixelSize: 16
                            font.bold: true
                            color: Kirigami.Theme.highlightColor
                        }

                        QQC2.Label {
                            text: "(via ALSA - hardware support limited)"
                            font.pixelSize: 11
                            color: Kirigami.Theme.disabledTextColor
                        }
                    }

                    // Gain slider row
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        QQC2.Label {
                            text: "Gain"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                            Layout.preferredWidth: 50
                        }

                        QQC2.Slider {
                            id: gainSlider
                            Layout.fillWidth: true
                            from: 0
                            to: 1
                            value: devicePage.localGainValue
                            enabled: controller.device_connected && !controller.mic_muted

                            onMoved: {
                                devicePage.localGainValue = value
                                controller.set_mic_gain_value(value)
                            }
                        }

                        QQC2.Label {
                            text: Math.round(devicePage.localGainValue * 100) + "%"
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                            Layout.preferredWidth: 40
                            horizontalAlignment: Text.AlignRight
                        }
                    }

                    // Mute button row
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        QQC2.Label {
                            text: "Mute"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                            Layout.preferredWidth: 50
                        }

                        QQC2.Button {
                            id: muteButton
                            Layout.preferredWidth: 100
                            Layout.preferredHeight: 36
                            text: controller.mic_muted ? "UNMUTE" : "MUTE"
                            enabled: controller.device_connected

                            onClicked: controller.toggle_mic_mute()

                            background: Rectangle {
                                color: controller.mic_muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.backgroundColor
                                radius: 6
                                border.color: controller.mic_muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.highlightColor
                                border.width: 1
                            }

                            contentItem: Text {
                                text: muteButton.text
                                font.pixelSize: 12
                                font.bold: true
                                color: Kirigami.Theme.textColor
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                            }
                        }

                        Rectangle {
                            visible: controller.mic_muted
                            width: 80
                            height: 28
                            radius: 4
                            color: Qt.rgba(Kirigami.Theme.negativeTextColor.r, Kirigami.Theme.negativeTextColor.g, Kirigami.Theme.negativeTextColor.b, 0.2)

                            RowLayout {
                                anchors.centerIn: parent
                                spacing: 4

                                Rectangle {
                                    width: 6
                                    height: 6
                                    radius: 3
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
                                    font.pixelSize: 10
                                    font.bold: true
                                    color: Kirigami.Theme.negativeTextColor
                                }
                            }
                        }

                        Item { Layout.fillWidth: true }
                    }
                }
            }

            // Monitor Output Section
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                implicitHeight: outputColumn.implicitHeight + 40
                color: Kirigami.Theme.alternateBackgroundColor
                radius: 12

                ColumnLayout {
                    id: outputColumn
                    anchors.fill: parent
                    anchors.margins: 20
                    spacing: 12

                    QQC2.Label {
                        text: "Monitor Output"
                        font.pixelSize: 16
                        font.bold: true
                        color: Kirigami.Theme.highlightColor
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        QQC2.Label {
                            text: "Device"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                            Layout.preferredWidth: 50
                        }

                        QQC2.ComboBox {
                            id: outputDeviceCombo
                            Layout.fillWidth: true
                            Layout.maximumWidth: 400
                            Layout.preferredHeight: 36
                            model: controller.output_device_count

                            Component.onCompleted: updateCurrentIndex()

                            function updateCurrentIndex() {
                                for (let i = 0; i < controller.output_device_count; i++) {
                                    if (controller.output_device_name(i) === controller.monitor_output) {
                                        currentIndex = i
                                        return
                                    }
                                }
                            }

                            Connections {
                                target: controller
                                function onMonitor_outputChanged() {
                                    outputDeviceCombo.updateCurrentIndex()
                                }
                                function onOutput_device_countChanged() {
                                    outputDeviceCombo.updateCurrentIndex()
                                }
                            }

                            displayText: controller.output_device_description(currentIndex) || "Select output..."

                            delegate: QQC2.ItemDelegate {
                                required property int index
                                width: outputDeviceCombo.width
                                text: controller.output_device_description(index)
                                highlighted: outputDeviceCombo.highlightedIndex === index
                            }

                            onActivated: (index) => {
                                let deviceName = controller.output_device_name(index)
                                if (deviceName && deviceName !== "") {
                                    controller.set_monitor_output_device(deviceName)
                                }
                            }
                        }

                        Item { Layout.fillWidth: true }
                    }
                }
            }

            // Audio Info Section
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                implicitHeight: audioColumn.implicitHeight + 40
                color: Kirigami.Theme.alternateBackgroundColor
                radius: 12

                ColumnLayout {
                    id: audioColumn
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 20
                    spacing: 8

                    QQC2.Label {
                        text: "Audio Configuration"
                        font.pixelSize: 16
                        font.bold: true
                        color: Kirigami.Theme.highlightColor
                    }

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        columnSpacing: 16
                        rowSpacing: 6

                        QQC2.Label {
                            text: "Sample Rate"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                        }
                        QQC2.Label {
                            text: "48000 Hz"
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        QQC2.Label {
                            text: "Bit Depth"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                        }
                        QQC2.Label {
                            text: "24-bit"
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        QQC2.Label {
                            text: "Channels"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                        }
                        QQC2.Label {
                            text: "Stereo (2ch)"
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        QQC2.Label {
                            text: "Latency"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                        }
                        QQC2.Label {
                            text: "Managed by PipeWire"
                            font.pixelSize: 13
                            color: Kirigami.Theme.disabledTextColor
                        }
                    }
                }
            }

            // Footer note
            QQC2.Label {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                text: "Note: Mic gain uses ALSA and may not work on all systems. Audio routing is managed by PipeWire."
                font.pixelSize: 11
                color: Kirigami.Theme.disabledTextColor
                wrapMode: Text.WordWrap
            }

            // Bottom margin
            Item { Layout.preferredHeight: 24 }
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
