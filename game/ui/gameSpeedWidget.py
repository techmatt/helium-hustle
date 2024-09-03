from __future__ import annotations

from functools import partial
from typing import Dict, List, NamedTuple

from PyQt6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                             QScrollArea, QFrame, QPushButton)
from PyQt6.QtCore import Qt, QSize
from PyQt6.QtGui import QFont, QIcon

from game.util.styleSheets import StyleSheets

class GameSpeedWidget(QWidget):
    def __init__(self, gameUI):
        super().__init__()
        self.gameUI = gameUI
        self.initUI()

    def initUI(self):
        layout = QHBoxLayout()
        self.setLayout(layout)

        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(2)

        self.speeds = [0, 1, 3, 10, 50, 200]
        self.iconNames = ['pause', '1x', '3x', '10x', '50x', '200x']
        self.buttons : Dict[int, QPushButton] = {}

        layout.addStretch()
        
        for speed, icon in zip(self.speeds, self.iconNames):
            button = self.gameUI.makeIconButton(f'icons/gameSpeed/{icon}.png', 56, 56)
            button.setToolTip(f'{speed}x speed')
            button.clicked.connect(partial(self.setGameSpeed, speed))
            self.buttons[speed] = button
            layout.addWidget(button)
        
        layout.addStretch()
        
        self.reloadIcons()

    def reloadIcons(self):
        for speed, icon in zip(self.speeds, self.iconNames):
            button = self.buttons[speed]
            selectedStr = 'Sel' if self.gameUI.gameSpeed == speed else ''
            button.setIcon(QIcon(f'icons/gameSpeed{selectedStr}/{icon}.png'))
            
    def setGameSpeed(self, speed):
        print(f"Setting game speed to {speed}x")
        self.gameUI.gameSpeed = speed
        self.reloadIcons()
        
