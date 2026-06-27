import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Layouts

// TODO: make popup appear on top if it's below the screen
Rectangle {
    Layout.fillHeight: true
    Layout.fillWidth: true

    HorizontalHeaderView {
        id: horizontalHeader

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        clip: true
        syncView: tableView

        delegate: Rectangle {
            implicitWidth: 100
            implicitHeight: 50
            Text {
                color: Colors.textMuted
                font.weight: 600
                text: display
            }
        }
    }
    TableView {
        id: tableView

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: horizontalHeader.bottom
        clip: true
        model: transactionTableModel

        delegate: TableViewDelegate {
            id: tableCell
            implicitWidth: 100
            implicitHeight: 50

            background: Item {
                Rectangle {
                    anchors.fill: parent
                    border.color: Colors.neutral50
                    border.width: 1
                    // anchors.margins: 10
                }
                Rectangle {
                    anchors.fill: parent
                    border.color: "darkblue"
                    border.width: tableCell.current ? 2 : 0
                    color: "transparent"
                }
            }

            TableView.editDelegate: DelegateChooser{

                DelegateChoice {
                    column: 1
                    delegate:  ComboBox{
                        id: combobox
                        anchors.fill: parent
                        model: accountModel

                        delegate: ItemDelegate {
                            id: delegate
                            required property var model
                            required property int index
                            width: combobox.width

                            contentItem: Text {
                                text: delegate.model[combobox.textRole]
                            }

                            onClicked: {
                                transactionTableModel.setAccount("A1","T1")
                            }
                        }
                    }

                }
            }

            contentItem: Item {
                visible: !tableCell.editing
                RowLayout {
                    anchors.fill: parent

                    CheckBox {
                        id: checkBox
                        checked: false
                        visible: tableCell.column === 0
                    }

                    Text {
                        Layout.leftMargin: 4
                        color: tableCell.selected ? "white" : "black"
                        text: tableCell.model.display
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }
}
