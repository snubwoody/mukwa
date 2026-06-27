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

        TransactionTable {}
    }
}
