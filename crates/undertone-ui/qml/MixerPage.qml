import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: mixerPage
    color: Kirigami.Theme.backgroundColor

    required property var controller

    // Channel brand colors for visual differentiation
    readonly property var channelColors: ({
        "system": "#e94560",
        "voice": "#f59e0b",
        "music": "#10b981",
        "browser": "#3b82f6",
        "game": "#8b5cf6"
    })

    function getChannelColor(channelName) {
        return channelColors[channelName] || Kirigami.Theme.highlightColor
    }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 8

        // Channel strips
        Repeater {
            model: controller.channel_count

            ChannelStrip {
                id: strip
                required property int index

                channelName: controller.channel_name(index)
                displayName: controller.channel_display_name(index)
                volume: controller.channel_volume(index)
                muted: controller.channel_muted(index)
                channelColor: mixerPage.getChannelColor(channelName)

                onVolumeAdjusted: (newVolume) => {
                    controller.set_channel_volume(channelName, newVolume)
                }

                onMuteToggled: {
                    controller.toggle_channel_mute(channelName)
                }
            }
        }

        // Spacer
        Item { Layout.fillWidth: true }

        // Master section
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 80
            color: Kirigami.Theme.alternateBackgroundColor
            radius: 8

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8

                QQC2.Label {
                    Layout.alignment: Qt.AlignHCenter
                    text: "Master"
                    font.pixelSize: 12
                    font.bold: true
                    color: Kirigami.Theme.textColor
                }

                Item { Layout.fillHeight: true }

                // Master volume display (placeholder)
                QQC2.Label {
                    Layout.alignment: Qt.AlignHCenter
                    text: "100%"
                    font.pixelSize: 14
                    color: Kirigami.Theme.highlightColor
                }

                Item { Layout.fillHeight: true }
            }
        }
    }

    // Empty state when no channels
    Column {
        anchors.centerIn: parent
        visible: controller.channel_count === 0
        spacing: 16

        Kirigami.Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            source: "audio-volume-high"
            width: 64
            height: 64
            color: Kirigami.Theme.disabledTextColor
        }

        QQC2.Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "No channels available"
            color: Kirigami.Theme.textColor
            font.pixelSize: 18
        }

        QQC2.Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: controller.connected ? "Waiting for daemon to create channels..." : "Connect to daemon to see channels"
            color: Kirigami.Theme.disabledTextColor
            font.pixelSize: 12
        }
    }
}
