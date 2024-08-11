
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
        return f"{rState.info.name}:  (+)"
    
    def initUI(self):
        self.resourceList = QVBoxLayout()
        self.resourceList.setSpacing(0)
        self.resourceList.setContentsMargins(0, 0, 0, 0)
        self.setLayout(self.resourceList)

        self.rValueLabels : Dict[str, QLabel] = {}
        self.rIncomeLabels : Dict[str, QLabel] = {}
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
        
            rLabelName = QLabel(f"{rState.info.name}:")
            rLabelValue = QLabel("")
            rLabelIncome = QLabel("")
            self.rValueLabels[rState.info.name] = rLabelValue
            self.rIncomeLabels[rState.info.name] = rLabelIncome
            
            rLabelName.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
            rLabelValue.setAlignment(Qt.AlignmentFlag.AlignCenter)
            rLabelIncome.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)
            
            rLayout.addWidget(iconLabel)
            rLayout.addWidget(rLabelName)
            rLayout.addWidget(rLabelValue)
            rLayout.addWidget(rLabelIncome)
            
            self.resourceList.addWidget(rWidget)
            
        self.updateLabels()
            
    def updateLabels(self):
        for rState in self.gameUI.state.resources.values():
            rLabelValue = self.rValueLabels[rState.info.name]
            rLabelIncome = self.rIncomeLabels[rState.info.name]
            rLabelValue.setText(f"{rState.count} / {rState.storage}")
            rLabelIncome.setText(f"{rState.income} /s")
