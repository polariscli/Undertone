import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: channelStrip
    Layout.fillHeight: true
    Layout.preferredWidth: 64
    color: Kirigami.Theme.alternateBackgroundColor
    radius: 8

    // Properties
    property string channelName: ""
    property string displayName: ""
    property real volume: 1.0
    property bool muted: false
    property real levelLeft: 0.0
    property real levelRight: 0.0
    property color channelColor: Kirigami.Theme.highlightColor  // Brand color for this channel

    // Signals
    signal volumeAdjusted(real newVolume)
    signal muteToggled()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 4

        // Channel name
        QQC2.Label {
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
                color: Kirigami.Theme.backgroundColor
                radius: 2

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: Math.min(levelLeft, 1.0) * parent.height
                    radius: 2
                    color: levelLeft > 0.9 ? Kirigami.Theme.negativeTextColor : (levelLeft > 0.7 ? Kirigami.Theme.neutralTextColor : Kirigami.Theme.positiveTextColor)

                    Behavior on height {
                        NumberAnimation { duration: 50 }
                    }
                }
            }

            // Volume slider
            QQC2.Slider {
                id: volumeSlider
                orientation: Qt.Vertical
                from: 0
                to: 1
                value: channelStrip.volume
                enabled: !channelStrip.muted
                Layout.fillHeight: true

                onMoved: {
                    channelStrip.volumeAdjusted(value)
                }

                background: Rectangle {
                    x: volumeSlider.leftPadding + volumeSlider.availableWidth / 2 - width / 2
                    y: volumeSlider.topPadding
                    width: 4
                    height: volumeSlider.availableHeight
                    radius: 2
                    color: Kirigami.Theme.backgroundColor

                    Rectangle {
                        width: parent.width
                        height: (1 - volumeSlider.visualPosition) * parent.height
                        y: parent.height - height
                        radius: 2
                        color: channelStrip.muted ? Kirigami.Theme.disabledTextColor : channelColor
                        opacity: channelStrip.muted ? 0.5 : 1.0
                    }
                }

                handle: Rectangle {
                    x: volumeSlider.leftPadding + volumeSlider.availableWidth / 2 - width / 2
                    y: volumeSlider.topPadding + volumeSlider.visualPosition * (volumeSlider.availableHeight - height)
                    width: 14
                    height: 6
                    radius: 2
                    color: volumeSlider.pressed ? Kirigami.Theme.textColor : (channelStrip.muted ? Kirigami.Theme.disabledTextColor : channelColor)
                    opacity: channelStrip.muted ? 0.5 : 1.0
                }
            }

            // Right VU meter
            Rectangle {
                Layout.fillHeight: true
                Layout.preferredWidth: 4
                color: Kirigami.Theme.backgroundColor
                radius: 2

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: Math.min(levelRight, 1.0) * parent.height
                    radius: 2
                    color: levelRight > 0.9 ? Kirigami.Theme.negativeTextColor : (levelRight > 0.7 ? Kirigami.Theme.neutralTextColor : Kirigami.Theme.positiveTextColor)

                    Behavior on height {
                        NumberAnimation { duration: 50 }
                    }
                }
            }
        }

        // Volume value label
        QQC2.Label {
            text: Math.round(channelStrip.volume * 100) + "%"
            font.pixelSize: 10
            color: channelStrip.muted ? Kirigami.Theme.disabledTextColor : Kirigami.Theme.textColor
            Layout.alignment: Qt.AlignHCenter
        }

        // Mute button
        QQC2.Button {
            Layout.preferredWidth: 40
            Layout.preferredHeight: 24
            Layout.alignment: Qt.AlignHCenter
            flat: true

            onClicked: channelStrip.muteToggled()

            background: Rectangle {
                color: channelStrip.muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.backgroundColor
                radius: 4
            }

            contentItem: Text {
                text: "M"
                font.pixelSize: 11
                font.bold: true
                color: channelStrip.muted ? Kirigami.Theme.textColor : Kirigami.Theme.disabledTextColor
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }
    }
}
