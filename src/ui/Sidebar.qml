import QtQuick
import QtQuick.Window
import QtQuick.Layouts
import QtQuick.Controls

// TODO: add focus border color for button
Rectangle {
    Layout.alignment: Qt.AlignTop
    Layout.fillHeight: true
    Layout.fillWidth: true
    Layout.maximumWidth: 250

    Rectangle {
        id: leftBorder
        width: 1
        color: Colors.neutral50
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
    }

    ColumnLayout {
        anchors.right: leftBorder.left
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        spacing: 0

        Text {
            text: qsTr("Spending")
        }

        Text {
            text: qsTr("Analytics")
        }

        RowLayout {
            Layout.fillWidth: true

            Text {
                font.weight: 600
                text: qsTr("Accounts")
            }
            Item {
                Layout.fillWidth: true
            }
            Text {
                font.weight: 600
                text: "K0.00"
            }
        }

        ListView {
            id: listView

            Layout.fillWidth: true
            implicitHeight: 200
            model: accountModel

            delegate: RowLayout {
                width: ListView.view.width

                Text {
                    text: display
                }
                Item {
                    Layout.fillWidth: true
                }
                Text {
                    text: "K0.00"
                }
            }
        }

        Button {
            id: control

            Layout.fillWidth: true
            text: qsTr("Add account")

            background: Rectangle {
                color: Colors.neutral50
                radius: 2
            }
            contentItem: Text {
                color: Colors.textBody
                horizontalAlignment: Text.AlignHCenter
                text: control.text
            }

            onClicked: popup.open()

            Popup {
                id: popup

                closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent
                dim: false
                focus: true
                implicitHeight: 200
                implicitWidth: control.width
                modal: true
                x: control.x
                y: control.y + control.height

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 8

                    TextField {
                        id: accountName

                        placeholderText: qsTr("Account name")
                    }
                    TextField {
                        id: startingBalance

                        placeholderText: qsTr("Starting balance")
                    }
                    Button {
                        text: qsTr("Confirm")
                        Layout.fillWidth: true

                        background: Rectangle {
                            // color: "#FFFFFF"
                            color: Colors.neutral50
                            radius: 2
                        }
                        contentItem: Text {
                            color: Colors.textBody
                            horizontalAlignment: Text.AlignHCenter
                            text: control.text
                        }

                        onClicked: {
                            accountModel.addAccount(accountName.text, parseFloat(startingBalance.text) || 0);

                            accountName.clear();
                            startingBalance.clear();

                            popup.close();
                        }
                    }
                }
            }
        }

    }
}
