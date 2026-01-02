import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: mixerPage
    color: Kirigami.Theme.backgroundColor

    required property var controller

    // Channel brand colors for visual differentiation (kept hardcoded per design)
    readonly property var channelColors: ["#e94560", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6"]

    // Channel strip container
    RowLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 8

        // Channel strips using index-based access
        Repeater {
            model: controller.channel_count

            ChannelStrip {
                required property int index

                channelName: controller.channel_name(index)
                displayName: controller.channel_display_name(index)
                volume: controller.channel_volume(index)
                muted: controller.channel_muted(index)
                levelLeft: 0.0  // TODO: Get from controller
                levelRight: 0.0
                channelColor: mixerPage.channelColors[index % mixerPage.channelColors.length]

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
                    text: "Master"
                    font.pixelSize: 12
                    font.bold: true
                    color: Kirigami.Theme.highlightColor
                    Layout.alignment: Qt.AlignHCenter
                }

                // Master volume slider (vertical)
                // TODO: Master volume is not yet implemented in daemon.
                // This slider is currently non-functional.
                QQC2.Slider {
                    id: masterSlider
                    orientation: Qt.Vertical
                    from: 0
                    to: 1
                    value: 1.0
                    Layout.fillHeight: true
                    Layout.alignment: Qt.AlignHCenter
                    enabled: false  // Disabled until daemon support is added

                    background: Rectangle {
                        x: masterSlider.leftPadding + masterSlider.availableWidth / 2 - width / 2
                        y: masterSlider.topPadding
                        width: 4
                        height: masterSlider.availableHeight
                        radius: 2
                        color: Kirigami.Theme.backgroundColor

                        Rectangle {
                            width: parent.width
                            height: masterSlider.visualPosition * parent.height
                            y: parent.height - height
                            radius: 2
                            color: Kirigami.Theme.highlightColor
                        }
                    }

                    handle: Rectangle {
                        x: masterSlider.leftPadding + masterSlider.availableWidth / 2 - width / 2
                        y: masterSlider.topPadding + masterSlider.visualPosition * (masterSlider.availableHeight - height)
                        width: 16
                        height: 8
                        radius: 2
                        color: masterSlider.pressed ? Kirigami.Theme.textColor : Kirigami.Theme.highlightColor
                    }
                }

                // Volume value
                QQC2.Label {
                    text: Math.round(masterSlider.value * 100) + "%"
                    font.pixelSize: 11
                    color: Kirigami.Theme.disabledTextColor
                    Layout.alignment: Qt.AlignHCenter
                }

                // Master mute button
                // TODO: Master mute is not yet implemented in daemon.
                QQC2.Button {
                    Layout.preferredWidth: 48
                    Layout.preferredHeight: 32
                    Layout.alignment: Qt.AlignHCenter
                    flat: true
                    text: "M"
                    font.bold: true
                    enabled: false  // Disabled until daemon support is added

                    background: Rectangle {
                        color: enabled ? Kirigami.Theme.backgroundColor : Kirigami.Theme.alternateBackgroundColor
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: enabled ? Kirigami.Theme.textColor : Kirigami.Theme.disabledTextColor
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }

    // Empty state when no channels
    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        visible: controller.channel_count === 0
        icon.name: "audio-volume-muted"
        text: "No channels available"
        explanation: "Start the daemon to see audio channels"
    }
}
