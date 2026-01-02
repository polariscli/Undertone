import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: channelStrip
    Layout.fillHeight: true
    Layout.preferredWidth: 72
    color: Kirigami.Theme.alternateBackgroundColor
    radius: 8

    property string channelName: ""
    property string displayName: ""
    property real volume: 1.0
    property bool muted: false
    property real levelLeft: 0.0
    property real levelRight: 0.0
    property color channelColor: "#e94560"

    signal volumeAdjusted(real newVolume)
    signal muteToggled()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 4

        // Channel color indicator
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 4
            radius: 2
            color: channelStrip.muted ? Kirigami.Theme.disabledTextColor : channelStrip.channelColor
        }

        // Channel name
        QQC2.Label {
            Layout.fillWidth: true
            text: channelStrip.displayName || channelStrip.channelName
            font.pixelSize: 11
            font.bold: true
            color: channelStrip.muted ? Kirigami.Theme.disabledTextColor : Kirigami.Theme.textColor
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideRight
        }

        // Vertical fader
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            QQC2.Slider {
                id: volumeSlider
                anchors.centerIn: parent
                width: parent.height
                height: 32
                rotation: -90

                from: 0
                to: 1
                value: channelStrip.volume
                enabled: !channelStrip.muted

                onMoved: {
                    channelStrip.volumeAdjusted(value)
                }

                background: Rectangle {
                    x: volumeSlider.leftPadding
                    y: volumeSlider.topPadding + volumeSlider.availableHeight / 2 - height / 2
                    width: volumeSlider.availableWidth
                    height: 6
                    radius: 3
                    color: Kirigami.Theme.backgroundColor

                    Rectangle {
                        width: volumeSlider.visualPosition * parent.width
                        height: parent.height
                        radius: 3
                        color: volumeSlider.enabled ? channelStrip.channelColor : Kirigami.Theme.disabledTextColor
                    }
                }

                handle: Rectangle {
                    x: volumeSlider.leftPadding + volumeSlider.visualPosition * (volumeSlider.availableWidth - width)
                    y: volumeSlider.topPadding + volumeSlider.availableHeight / 2 - height / 2
                    width: 16
                    height: 16
                    radius: 8
                    color: volumeSlider.pressed ? Kirigami.Theme.textColor : (volumeSlider.enabled ? channelStrip.channelColor : Kirigami.Theme.disabledTextColor)
                    border.color: Kirigami.Theme.backgroundColor
                    border.width: 2
                }
            }
        }

        // Volume percentage
        QQC2.Label {
            Layout.fillWidth: true
            text: Math.round(channelStrip.volume * 100) + "%"
            font.pixelSize: 11
            color: channelStrip.muted ? Kirigami.Theme.disabledTextColor : Kirigami.Theme.textColor
            horizontalAlignment: Text.AlignHCenter
        }

        // Mute button
        QQC2.Button {
            Layout.fillWidth: true
            Layout.preferredHeight: 28
            text: channelStrip.muted ? "M" : "M"
            font.pixelSize: 12
            font.bold: true
            flat: true

            onClicked: channelStrip.muteToggled()

            background: Rectangle {
                color: channelStrip.muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.backgroundColor
                radius: 4
                border.color: channelStrip.muted ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.disabledTextColor
                border.width: 1
            }

            contentItem: Text {
                text: parent.text
                font: parent.font
                color: channelStrip.muted ? Kirigami.Theme.textColor : Kirigami.Theme.disabledTextColor
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }
    }
}
