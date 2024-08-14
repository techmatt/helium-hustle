
from __future__ import annotations

import sys
import os
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from enums import GameWindowMode
from gameDatabase import GameDatabase
from gameState import GameState
from styleSheets import StyleSheets

class IconButton(QPushButton):
    clicked = pyqtSignal(str)  # Custom signal to emit the button's name

    def __init__(self, name, iconPath, parent=None):
        super().__init__(parent)
        self.name = name
        self.setFixedSize(120, 100)  # Adjust size as needed

        layout = QVBoxLayout(self)
        layout.setSpacing(5)
        layout.setContentsMargins(5, 5, 5, 5)

        # Icon
        iconLabel = QLabel()
        pixmap = QPixmap(iconPath).scaled(
            80, 80, 
            Qt.AspectRatioMode.KeepAspectRatio, 
            Qt.TransformationMode.SmoothTransformation
        )
        iconLabel.setPixmap(pixmap)
        iconLabel.setAlignment(Qt.AlignmentFlag.AlignCenter)

        # Text
        textLabel = QLabel(name)
        textLabel.setAlignment(Qt.AlignmentFlag.AlignCenter)
        textLabel.setStyleSheet(StyleSheets.ICON_LIST_TEXT)

        layout.addWidget(iconLabel)
        layout.addWidget(textLabel)

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        if self.underMouse():
            painter.setBrush(QColor(200, 200, 200, 100))
        else:
            painter.setBrush(QColor(230, 230, 230, 100))

        painter.setPen(QColor(180, 180, 180))
        painter.drawRoundedRect(self.rect(), 10, 10)

    def enterEvent(self, event):
        self.update()

    def leaveEvent(self, event):
        self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.name)

class IconGrid(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        self.initUI()

    def initUI(self):
        self.grid = QGridLayout()
        self.setLayout(self.grid)

        icons = [
            ("Commands", "commands.png"),
            ("Buildings", "buildings.png"),
            ("Research", "research.png"),
            ("Achievements", "achievements.png"),
            ("Options", "options.png"),
            ("Exit", "exit.png")
        ]

        for index, (name, iconPath) in enumerate(icons):
            
            button = IconButton(name, 'icons/actionGrid/' + iconPath)
            button.clicked.connect(self.onButtonClicked)
            
            row = index // 3
            col = index % 3
            self.grid.addWidget(button, row, col)
            
    def onButtonClicked(self, name):
        if name == 'Commands':
            self.gameUI.mode = GameWindowMode.COMMANDS
            self.gameUI.makeMiddleFrame()
        if name == 'Buildings':
            self.gameUI.mode = GameWindowMode.BUILDINGS
            self.gameUI.makeMiddleFrame()
        if name == 'Exit':
            self.gameUI.triggerExit()
        #print(f"Clicked on {name}")
