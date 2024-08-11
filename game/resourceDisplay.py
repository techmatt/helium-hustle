
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

    def getResourceString(self, rState : ResourceState):
        return f"{rState.info.name}: {rState.count} / {rState.storage} (+{rState.income} / sec)"
    
    def initUI(self):
        self.resourceList = QVBoxLayout()
        self.resourceList.setSpacing(0)
        self.resourceList.setContentsMargins(0, 0, 0, 0)
        self.setLayout(self.resourceList)

        self.rLabels : Dict[str, QLabel] = {}
        for rState in self.gameUI.state.resources.values():
            #print('resource: ', r.info.name)
            rWidget = QWidget()
            rLayout = QHBoxLayout(rWidget)
            rLayout.setContentsMargins(0, 0, 0, 0)
            
            iconPath = 'icons/resources/' + rState.info.name + '.png'
            iconLabel = QLabel()
            pixmap = QPixmap(iconPath).scaled(
                32, 32, 
                Qt.AspectRatioMode.KeepAspectRatio, 
                Qt.TransformationMode.SmoothTransformation
            )
            iconLabel.setPixmap(pixmap)
            iconLabel.setFixedSize(32, 32)
        
            rLabel = QLabel(self.getResourceString(rState))
            self.rLabels[rState.info.name] = rLabel
            
            rLayout.addWidget(iconLabel)
            rLayout.addWidget(rLabel)
            
            self.resourceList.addWidget(rWidget)
            
    def updateLabels(self):
        for rState in self.gameUI.state.resources.values():
            rLabel = self.rLabels[rState.info.name]
            rText = self.getResourceString(rState)
            rLabel.setText(rText)
