import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: appsPage
    color: Kirigami.Theme.backgroundColor

    required property var controller

    // Channel brand colors for visual differentiation (kept hardcoded per design)
    readonly property var channelColors: ({
        "system": "#e94560",
        "voice": "#f59e0b",
        "music": "#10b981",
        "browser": "#3b82f6",
        "game": "#8b5cf6"
    })

    function getChannelColor(channel) {
        return channelColors[channel] || Kirigami.Theme.disabledTextColor
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

            QQC2.Label {
                text: "Active Applications"
                font.pixelSize: 18
                font.bold: true
                color: Kirigami.Theme.textColor
            }

            Item { Layout.fillWidth: true }

            QQC2.Button {
                text: "Refresh"
                icon.name: "view-refresh"
                flat: true
                onClicked: controller.refresh()
            }
        }

        // App list
        QQC2.ScrollView {
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
                    color: Kirigami.Theme.alternateBackgroundColor
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

                            QQC2.Label {
                                anchors.centerIn: parent
                                text: appName.charAt(0).toUpperCase()
                                font.pixelSize: 18
                                font.bold: true
                                color: Kirigami.Theme.textColor
                            }
                        }

                        // App info
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            QQC2.Label {
                                text: appName
                                font.pixelSize: 14
                                font.bold: true
                                color: Kirigami.Theme.textColor
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            QQC2.Label {
                                text: appBinary || "Unknown binary"
                                font.pixelSize: 11
                                color: Kirigami.Theme.disabledTextColor
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
                            color: Kirigami.Theme.backgroundColor

                            QQC2.Label {
                                anchors.centerIn: parent
                                text: "P"
                                font.pixelSize: 10
                                font.bold: true
                                color: Kirigami.Theme.positiveTextColor
                            }

                            QQC2.ToolTip.visible: persistentMouse.containsMouse
                            QQC2.ToolTip.text: "Persistent route"

                            MouseArea {
                                id: persistentMouse
                                anchors.fill: parent
                                hoverEnabled: true
                            }
                        }

                        // Channel selector with custom popup to avoid Breeze Overlay issues
                        QQC2.ComboBox {
                            id: channelCombo
                            Layout.preferredWidth: 120
                            model: controller.available_channels().split(",")
                            currentIndex: model.indexOf(appChannel)

                            onActivated: (idx) => {
                                controller.set_app_channel(appBinary || appName, model[idx])
                            }

                            background: Rectangle {
                                color: Kirigami.Theme.alternateBackgroundColor
                                radius: 4
                                border.color: appsPage.getChannelColor(appChannel)
                                border.width: 2
                            }

                            contentItem: RowLayout {
                                spacing: 8

                                Rectangle {
                                    Layout.leftMargin: 8
                                    width: 8
                                    height: 8
                                    radius: 4
                                    color: appsPage.getChannelColor(channelCombo.currentText)
                                }

                                Text {
                                    text: appsPage.getChannelDisplayName(channelCombo.currentText)
                                    color: Kirigami.Theme.textColor
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }

                            delegate: QQC2.ItemDelegate {
                                width: channelCombo.width
                                height: 32

                                required property string modelData
                                required property int index

                                contentItem: RowLayout {
                                    Rectangle {
                                        Layout.leftMargin: 8
                                        width: 8
                                        height: 8
                                        radius: 4
                                        color: appsPage.getChannelColor(modelData)
                                    }
                                    Text {
                                        text: appsPage.getChannelDisplayName(modelData)
                                        color: Kirigami.Theme.textColor
                                        verticalAlignment: Text.AlignVCenter
                                        Layout.fillWidth: true
                                    }
                                }

                                background: Rectangle {
                                    color: parent.highlighted ? appsPage.getChannelColor(modelData) : Kirigami.Theme.alternateBackgroundColor
                                }
                            }

                            popup: QQC2.Popup {
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
                                    color: Kirigami.Theme.alternateBackgroundColor
                                    border.color: Kirigami.Theme.backgroundColor
                                    radius: 4
                                }
                            }
                        }
                    }
                }
            }
        }

        // Empty state
        Kirigami.PlaceholderMessage {
            visible: controller.app_count === 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            icon.name: "applications-multimedia"
            text: "No audio applications detected"
            explanation: "Play some audio to see apps here"
        }

        // Route rules section
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 150
            color: Kirigami.Theme.alternateBackgroundColor
            radius: 8

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                RowLayout {
                    Layout.fillWidth: true

                    QQC2.Label {
                        text: "Default Routes"
                        font.pixelSize: 14
                        font.bold: true
                        color: Kirigami.Theme.disabledTextColor
                    }

                    Item { Layout.fillWidth: true }

                    QQC2.Label {
                        text: "Rules are applied automatically"
                        font.pixelSize: 11
                        color: Kirigami.Theme.disabledTextColor
                    }
                }

                // Quick reference of default routes
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 24
                    rowSpacing: 4

                    QQC2.Label { text: "Discord, Zoom, Teams"; font.pixelSize: 11; color: Kirigami.Theme.disabledTextColor }
                    QQC2.Label { text: "-> Voice"; font.pixelSize: 11; color: channelColors["voice"]; font.bold: true }

                    QQC2.Label { text: "Spotify, Rhythmbox"; font.pixelSize: 11; color: Kirigami.Theme.disabledTextColor }
                    QQC2.Label { text: "-> Music"; font.pixelSize: 11; color: channelColors["music"]; font.bold: true }

                    QQC2.Label { text: "Firefox, Chrome"; font.pixelSize: 11; color: Kirigami.Theme.disabledTextColor }
                    QQC2.Label { text: "-> Browser"; font.pixelSize: 11; color: channelColors["browser"]; font.bold: true }

                    QQC2.Label { text: "Steam"; font.pixelSize: 11; color: Kirigami.Theme.disabledTextColor }
                    QQC2.Label { text: "-> Game"; font.pixelSize: 11; color: channelColors["game"]; font.bold: true }
                }

                Item { Layout.fillHeight: true }
            }
        }
    }
}
