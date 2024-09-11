
from __future__ import annotations

import sys
import os
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.util.enums import GameWindowMode
from game.database.gameDatabase import GameDatabase
from game.core.gameState import GameState
from game.util.styleSheets import StyleSheets

class IconButton(QPushButton):
    clicked = pyqtSignal(str)  # Custom signal to emit the button's name

    def __init__(self, name : str, iconPath : str, gameUI : GameUI):
        super().__init__()
        self.name = name
        self.setFixedSize(120, 80)  # Adjust size as needed

        layout = QVBoxLayout(self)
        layout.setSpacing(0)
        layout.setContentsMargins(0, 0, 0, 0)

        # Icon
        iconLabel = QLabel()
        iconLabel.setPixmap(gameUI.pixmapCache.getPixmap(iconPath, 64, 64))
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

class MainMenuWidget(QWidget):
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
            ("Projects", "projects.png"),
            ("Ideologies", "ideologies.png"),
            ("Adversaries", "adversaries.png"),
            ("Stats", "stats.png"),
            ("Achievements", "achievements.png"),
            ("Options", "options.png"),
            ("Exit", "exit.png")
        ]

        for index, (name, iconPath) in enumerate(icons):
            
            button = IconButton(name, 'icons/mainMenu/' + iconPath, self.gameUI)
            button.clicked.connect(self.onButtonClicked)
            
            row = index // 3
            col = index % 3
            self.grid.addWidget(button, row, col)
            
    def onButtonClicked(self, name):
        if name == 'Commands':
            self.gameUI.mode = GameWindowMode.COMMANDS
            self.gameUI.majorUIUpdate()
        if name == 'Buildings':
            self.gameUI.mode = GameWindowMode.BUILDINGS
            self.gameUI.majorUIUpdate()
        if name == 'Research':
            self.gameUI.mode = GameWindowMode.RESEARCH
            self.gameUI.majorUIUpdate()
        if name == 'Projects':
            self.gameUI.mode = GameWindowMode.PROJECTS
            self.gameUI.majorUIUpdate()
        if name == 'Ideologies':
            self.gameUI.mode = GameWindowMode.IDEOLOGIES
            self.gameUI.majorUIUpdate()
        if name == 'Stats':
            self.gameUI.mode = GameWindowMode.STATS
            self.gameUI.majorUIUpdate()
        if name == 'Achievements':
            self.gameUI.mode = GameWindowMode.ACHIEVEMENTS
            self.gameUI.majorUIUpdate()
        if name == 'Options':
            self.gameUI.mode = GameWindowMode.OPTIONS
            self.gameUI.majorUpdate()
        if name == 'Exit':
            self.gameUI.triggerExit()
        #print(f"Clicked on {name}")
