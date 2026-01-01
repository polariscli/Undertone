import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: appsPage
    color: "#1a1a2e"

    required property var controller

    // Channel colors for visual consistency
    readonly property var channelColors: ({
        "system": "#e94560",
        "voice": "#f59e0b",
        "music": "#10b981",
        "browser": "#3b82f6",
        "game": "#8b5cf6"
    })

    function getChannelColor(channel) {
        return channelColors[channel] || "#64748b"
    }

    function getChannelDisplayName(channel) {
        const names = {
            "system": "System",
            "voice": "Voice",
            "music": "Music",
            "browser": "Browser",
            "game": "Game"
        }
        return names[channel] || channel
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 16

        // Header
        RowLayout {
            Layout.fillWidth: true
            spacing: 16

            Label {
                text: "Active Applications"
                font.pixelSize: 18
                font.bold: true
                color: "#ffffff"
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "Refresh"
                flat: true
                onClicked: controller.refresh()

                background: Rectangle {
                    color: parent.hovered ? "#0f3460" : "#16213e"
                    radius: 4
                }

                contentItem: Text {
                    text: parent.text
                    color: "#94a3b8"
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }

        // App list
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: appList
                model: controller.app_count
                spacing: 8

                delegate: Rectangle {
                    required property int index

                    width: appList.width
                    height: 64
                    color: "#16213e"
                    radius: 8

                    property string appName: controller.app_name(index)
                    property string appBinary: controller.app_binary(index)
                    property string appChannel: controller.app_channel(index)
                    property bool isPersistent: controller.app_persistent(index)

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 16

                        // App icon placeholder
                        Rectangle {
                            Layout.preferredWidth: 40
                            Layout.preferredHeight: 40
                            radius: 8
                            color: appsPage.getChannelColor(appChannel)

                            Label {
                                anchors.centerIn: parent
                                text: appName.charAt(0).toUpperCase()
                                font.pixelSize: 18
                                font.bold: true
                                color: "#ffffff"
                            }
                        }

                        // App info
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Label {
                                text: appName
                                font.pixelSize: 14
                                font.bold: true
                                color: "#ffffff"
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            Label {
                                text: appBinary || "Unknown binary"
                                font.pixelSize: 11
                                color: "#64748b"
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                        }

                        // Persistent indicator
                        Rectangle {
                            visible: isPersistent
                            Layout.preferredWidth: 20
                            Layout.preferredHeight: 20
                            radius: 10
                            color: "#0f3460"

                            Label {
                                anchors.centerIn: parent
                                text: "P"
                                font.pixelSize: 10
                                font.bold: true
                                color: "#10b981"
                            }

                            ToolTip.visible: persistentMouse.containsMouse
                            ToolTip.text: "Persistent route"

                            MouseArea {
                                id: persistentMouse
                                anchors.fill: parent
                                hoverEnabled: true
                            }
                        }

                        // Channel selector
                        ComboBox {
                            id: channelCombo
                            Layout.preferredWidth: 120
                            model: controller.available_channels().split(",")
                            currentIndex: model.indexOf(appChannel)

                            onActivated: (idx) => {
                                controller.set_app_channel(appBinary || appName, model[idx])
                            }

                            background: Rectangle {
                                color: "#0f3460"
                                radius: 4
                                border.color: appsPage.getChannelColor(appChannel)
                                border.width: 2
                            }

                            contentItem: Text {
                                leftPadding: 8
                                text: appsPage.getChannelDisplayName(channelCombo.currentText)
                                color: "#ffffff"
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                            }

                            delegate: ItemDelegate {
                                width: channelCombo.width
                                height: 32

                                required property string modelData
                                required property int index

                                contentItem: Text {
                                    text: appsPage.getChannelDisplayName(modelData)
                                    color: "#ffffff"
                                    verticalAlignment: Text.AlignVCenter
                                }

                                background: Rectangle {
                                    color: parent.highlighted ? appsPage.getChannelColor(modelData) : "#16213e"
                                }
                            }

                            popup: Popup {
                                y: channelCombo.height
                                width: channelCombo.width
                                implicitHeight: contentItem.implicitHeight
                                padding: 1

                                contentItem: ListView {
                                    clip: true
                                    implicitHeight: contentHeight
                                    model: channelCombo.popup.visible ? channelCombo.delegateModel : null
                                    currentIndex: channelCombo.highlightedIndex
                                }

                                background: Rectangle {
                                    color: "#16213e"
                                    border.color: "#0f3460"
                                    radius: 4
                                }
                            }
                        }
                    }
                }
            }
        }

        // Empty state
        Label {
            visible: controller.app_count === 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            text: "No audio applications detected\n\nPlay some audio to see apps here"
            color: "#64748b"
            font.pixelSize: 16
        }

        // Route rules section
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 150
            color: "#16213e"
            radius: 8

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                RowLayout {
                    Layout.fillWidth: true

                    Label {
                        text: "Default Routes"
                        font.pixelSize: 14
                        font.bold: true
                        color: "#94a3b8"
                    }

                    Item { Layout.fillWidth: true }

                    Label {
                        text: "Rules are applied automatically"
                        font.pixelSize: 11
                        color: "#64748b"
                    }
                }

                // Quick reference of default routes
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 24
                    rowSpacing: 4

                    Label { text: "Discord, Zoom, Teams"; font.pixelSize: 11; color: "#64748b" }
                    Label { text: "-> Voice"; font.pixelSize: 11; color: channelColors["voice"]; font.bold: true }

                    Label { text: "Spotify, Rhythmbox"; font.pixelSize: 11; color: "#64748b" }
                    Label { text: "-> Music"; font.pixelSize: 11; color: channelColors["music"]; font.bold: true }

                    Label { text: "Firefox, Chrome"; font.pixelSize: 11; color: "#64748b" }
                    Label { text: "-> Browser"; font.pixelSize: 11; color: channelColors["browser"]; font.bold: true }

                    Label { text: "Steam"; font.pixelSize: 11; color: "#64748b" }
                    Label { text: "-> Game"; font.pixelSize: 11; color: channelColors["game"]; font.bold: true }
                }

                Item { Layout.fillHeight: true }
            }
        }
    }
}
