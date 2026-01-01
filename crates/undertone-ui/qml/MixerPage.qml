import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: mixerPage
    color: "#1a1a2e"

    required property var controller

    // Channel strip container
    RowLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 8

        // Channel strips using index-based access
        Repeater {
            model: controller.channelCount

            ChannelStrip {
                required property int index

                channelName: controller.channelName(index)
                displayName: controller.channelDisplayName(index)
                volume: controller.channelVolume(index)
                muted: controller.channelMuted(index)
                levelLeft: 0.0  // TODO: Get from controller
                levelRight: 0.0
                channelColor: getChannelColor(index)

                onVolumeAdjusted: (newVolume) => {
                    controller.setChannelVolume(channelName, newVolume)
                }

                onMuteToggled: {
                    controller.toggleChannelMute(channelName)
                }

                function getChannelColor(idx) {
                    const colors = ["#e94560", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6"]
                    return colors[idx % colors.length]
                }
            }
        }

        // Spacer
        Item { Layout.fillWidth: true }

        // Master section
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 80
            color: "#16213e"
            radius: 8

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8

                Label {
                    text: "Master"
                    font.pixelSize: 12
                    font.bold: true
                    color: "#e94560"
                    Layout.alignment: Qt.AlignHCenter
                }

                // Master volume slider (vertical)
                Slider {
                    id: masterSlider
                    orientation: Qt.Vertical
                    from: 0
                    to: 1
                    value: 1.0
                    Layout.fillHeight: true
                    Layout.alignment: Qt.AlignHCenter

                    background: Rectangle {
                        x: masterSlider.leftPadding + masterSlider.availableWidth / 2 - width / 2
                        y: masterSlider.topPadding
                        width: 4
                        height: masterSlider.availableHeight
                        radius: 2
                        color: "#0f3460"

                        Rectangle {
                            width: parent.width
                            height: masterSlider.visualPosition * parent.height
                            y: parent.height - height
                            radius: 2
                            color: "#e94560"
                        }
                    }

                    handle: Rectangle {
                        x: masterSlider.leftPadding + masterSlider.availableWidth / 2 - width / 2
                        y: masterSlider.topPadding + masterSlider.visualPosition * (masterSlider.availableHeight - height)
                        width: 16
                        height: 8
                        radius: 2
                        color: masterSlider.pressed ? "#ffffff" : "#e94560"
                    }
                }

                // Volume value
                Label {
                    text: Math.round(masterSlider.value * 100) + "%"
                    font.pixelSize: 11
                    color: "#94a3b8"
                    Layout.alignment: Qt.AlignHCenter
                }

                // Master mute button
                Button {
                    Layout.preferredWidth: 48
                    Layout.preferredHeight: 32
                    Layout.alignment: Qt.AlignHCenter
                    flat: true
                    text: "M"
                    font.bold: true

                    background: Rectangle {
                        color: "#0f3460"
                        radius: 4
                    }

                    contentItem: Text {
                        text: parent.text
                        color: "#ffffff"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }

    // Empty state when no channels
    Label {
        anchors.centerIn: parent
        visible: controller.channelCount === 0
        text: "No channels available"
        color: "#64748b"
        font.pixelSize: 18
    }
}
