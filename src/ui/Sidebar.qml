import QtQuick
import QtQuick.Window
import QtQuick.Layouts
import QtQuick.Controls

ColumnLayout {
    Layout.fillHeight: true
    Layout.fillWidth: true
    Layout.maximumWidth: 250
    Layout.alignment: Qt.AlignTop
    Layout.margins: 8
    spacing: 12

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
        implicitHeight: contentHeight
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
            y: control.y

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
                    Layout.fillWidth: true
                    text: qsTr("Confirm")

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
