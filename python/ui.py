from PyQt5.QtWidgets import QWidget,QApplication, QGridLayout, QPushButton, QTextBrowser, QLineEdit, QMainWindow, QLabel
from PyQt5.QtCore import QCoreApplication
import os
import sys

class MainWindow(QWidget):
    def __init__(self):
        super().__init__()
        self.initUI()

    def initUI(self):
        lang_apply = QPushButton('加载')
        analyze = QPushButton('分析')

        lang_path = QLineEdit()#输入语法
        t_path = QLineEdit()#终结符路径
        nt_path = QLineEdit()#非终结符路径
        lang = QLineEdit()#待分析语句

        lang_label = QLabel('语法文件路径')
        t_label = QLabel('终结符路径')
        nt_label = QLabel('非终结符路径')
        lg_label = QLabel('待分析字符串(以#结尾)')
        

        status = QTextBrowser()#状态窗口(终端)

        grid = QGridLayout()

        grid.addWidget(lang_label, 1, 0)
        grid.addWidget(lang_path, 1, 1)
        grid.addWidget(t_label, 2, 0)
        grid.addWidget(t_path, 2, 1)
        grid.addWidget(nt_label, 3, 0)
        grid.addWidget(nt_path, 3, 1)
        grid.addWidget(lang_apply, 4, 2)
        grid.addWidget(lang, 5, 1)
        grid.addWidget(lg_label, 5, 0)
        grid.addWidget(analyze, 5, 2)
        grid.addWidget(status, 6, 0, 10, 3)

        self.setLayout(grid)

        self.setWindowTitle('FF1语法分析')
        self.setGeometry(500, 300, 300, 300)
        self.show()

if __name__ == "__main__":
    #app = QCoreApplication(sys.argv)
    app = QApplication(sys.argv)
    win = MainWindow()

    sys.exit(app.exec_())



