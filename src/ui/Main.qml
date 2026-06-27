import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: root
    title: "Finance app"
    visible: true

    height: 750
    width: 750

    RowLayout {
        anchors.fill: parent

        Sidebar {}

        Rectangle {
            id: leftBorder
            implicitWidth: 1
            Layout.fillHeight: true
            color: Colors.neutral50
        }

        TransactionTable {}
    }
}
