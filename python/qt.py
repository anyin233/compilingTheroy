from PyQt5.QtWidgets import QApplication, QWidget, QPushButton, QToolTip, QMessageBox, QDesktopWidget, QGridLayout, QLabel
from PyQt5.QtCore import QCoreApplication
import sys

class Dialog(QWidget):
    def __init__(self):
        super().__init__()
        self.initUI()

    def initUI(self):
        #self.statusBar().showMessage("Ready")
        self.setToolTip('Hello world')

        btn = QPushButton('Button')
        btn.setToolTip('This is button')
        btn.resize(btn.sizeHint())
        btn.clicked.connect(QCoreApplication.instance().quit)
        
        label = QLabel('Hello World')
        grid = QGridLayout()
        grid.setSpacing(10)
        grid.addWidget(btn, 1, 0, 1, 1);
        grid.addWidget(label, 2, 1, 2, 3);

        self.setLayout(grid)
        self.setWindowTitle('Dialog')
        self.setGeometry(300, 300, 300, 300)
        self.center()
        self.show()

    def center(self):

        qr = self.frameGeometry()
        cp = QDesktopWidget().availableGeometry().center()
        qr.moveCenter(cp)
        self.move(qr.topLeft())

    def closeEvent(self, event):
        reply = QMessageBox.question(self, 'Question', 'Want quit?', QMessageBox.Yes | QMessageBox.No , QMessageBox.No)
        
        if reply == QMessageBox.Yes:
            event.accept()
        else:
            event.ignore()

if __name__ == "__main__":
    app = QApplication(sys.argv)

    ex = Dialog()
    #w = QWidget()
    #w.setWindowTitle("Hello world")
    #w.resize(300, 300)
    #w.move(300, 300)
    #w.show()

    sys.exit(app.exec_())