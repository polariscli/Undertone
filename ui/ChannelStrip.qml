import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: channelStrip
    Layout.fillHeight: true
    Layout.preferredWidth: 64
    color: "#16213e"
    radius: 8

    // Properties
    property string channelName: ""
    property string displayName: ""
    property real volume: 1.0
    property bool muted: false
    property real levelLeft: 0.0
    property real levelRight: 0.0
    property color channelColor: "#e94560"

    // Signals
    signal volumeChanged(real newVolume)
    signal muteToggled()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 4

        // Channel name
        Label {
            text: displayName
            font.pixelSize: 11
            font.bold: true
            color: channelColor
            elide: Text.ElideRight
            Layout.alignment: Qt.AlignHCenter
            Layout.maximumWidth: parent.width - 16
        }

        // VU meter and fader container
        RowLayout {
            Layout.fillHeight: true
            Layout.fillWidth: true
            spacing: 2

            // Left VU meter
            Rectangle {
                Layout.fillHeight: true
                Layout.preferredWidth: 4
                color: "#0f3460"
                radius: 2

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: Math.min(levelLeft, 1.0) * parent.height
                    radius: 2
                    color: levelLeft > 0.9 ? "#ef4444" : (levelLeft > 0.7 ? "#f59e0b" : "#4ade80")

                    Behavior on height {
                        NumberAnimation { duration: 50 }
                    }
                }
            }

            // Volume slider
            Slider {
                id: volumeSlider
                orientation: Qt.Vertical
                from: 0
                to: 1
                value: channelStrip.volume
                enabled: !channelStrip.muted
                Layout.fillHeight: true

                onMoved: {
                    channelStrip.volumeChanged(value)
                }

                background: Rectangle {
                    x: volumeSlider.leftPadding + volumeSlider.availableWidth / 2 - width / 2
                    y: volumeSlider.topPadding
                    width: 4
                    height: volumeSlider.availableHeight
                    radius: 2
                    color: "#0f3460"

                    Rectangle {
                        width: parent.width
                        height: (1 - volumeSlider.visualPosition) * parent.height
                        y: parent.height - height
                        radius: 2
                        color: channelStrip.muted ? "#64748b" : channelColor
                        opacity: channelStrip.muted ? 0.5 : 1.0
                    }
                }

                handle: Rectangle {
                    x: volumeSlider.leftPadding + volumeSlider.availableWidth / 2 - width / 2
                    y: volumeSlider.topPadding + volumeSlider.visualPosition * (volumeSlider.availableHeight - height)
                    width: 14
                    height: 6
                    radius: 2
                    color: volumeSlider.pressed ? "#ffffff" : (channelStrip.muted ? "#64748b" : channelColor)
                    opacity: channelStrip.muted ? 0.5 : 1.0
                }
            }

            // Right VU meter
            Rectangle {
                Layout.fillHeight: true
                Layout.preferredWidth: 4
                color: "#0f3460"
                radius: 2

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: Math.min(levelRight, 1.0) * parent.height
                    radius: 2
                    color: levelRight > 0.9 ? "#ef4444" : (levelRight > 0.7 ? "#f59e0b" : "#4ade80")

                    Behavior on height {
                        NumberAnimation { duration: 50 }
                    }
                }
            }
        }

        // Volume value label
        Label {
            text: Math.round(channelStrip.volume * 100) + "%"
            font.pixelSize: 10
            color: channelStrip.muted ? "#64748b" : "#94a3b8"
            Layout.alignment: Qt.AlignHCenter
        }

        // Mute button
        Button {
            Layout.preferredWidth: 40
            Layout.preferredHeight: 24
            Layout.alignment: Qt.AlignHCenter
            flat: true

            onClicked: channelStrip.muteToggled()

            background: Rectangle {
                color: channelStrip.muted ? "#ef4444" : "#0f3460"
                radius: 4
            }

            contentItem: Text {
                text: "M"
                font.pixelSize: 11
                font.bold: true
                color: channelStrip.muted ? "#ffffff" : "#94a3b8"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }
    }
}
