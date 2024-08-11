
from __future__ import annotations

import sys
import os
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import GameDatabase
from gameState import GameState

class ResourceDisplay(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        self.initUI()

    def initUI(self):
        self.resourceList = QVBoxLayout()
        self.setLayout(self.resourceList)

        self.resourceLabels = []
        for r in self.gameUI.state.resources.values():
            #print('resource: ', r.info.name)
            l = QLabel(f"{r.info.name}: {r.count} / {r.storage} (+{r.income} / sec)")
            self.resourceList.addWidget(l)
