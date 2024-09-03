
from __future__ import annotations

import sys
import os
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import GameDatabase
from game.core.gameState import GameState
from game.util.styleSheets import StyleSheets

class ResourceDisplayWidget(QWidget):
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
        
        self.addResourceWidget(self.gameUI.state.resources["Processors"])
        for rState in self.gameUI.state.resources.values():
            if rState.info.name != "Processors":
                self.addResourceWidget(rState)
            
        self.updateLabels()
        
    def addResourceWidget(self, rState):
        rWidget = QWidget()
        rLayout = QHBoxLayout(rWidget)
        rLayout.setContentsMargins(0, 0, 0, 0)
            
        iconPath = 'icons/resources/' + rState.info.name + '.png'
        iconLabel = self.gameUI.makeIconLabel(iconPath, 32, 32)
        
        rLabelName = QLabel(f"{rState.info.name}:")
        rLabelValue = QLabel("")
        rLabelIncome = QLabel("")
        self.rValueLabels[rState.info.name] = rLabelValue
        self.rIncomeLabels[rState.info.name] = rLabelIncome
            
        rLabelName.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        rLabelValue.setAlignment(Qt.AlignmentFlag.AlignCenter)
        rLabelIncome.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)
            
        rLabelName.setStyleSheet(StyleSheets.RESOURCE_LIST_TEXT)
        rLabelValue.setStyleSheet(StyleSheets.RESOURCE_LIST_TEXT)
        rLabelIncome.setStyleSheet(StyleSheets.RESOURCE_LIST_TEXT)
            
        rLayout.addWidget(iconLabel)
        
        if rState.info.name == "Processors":
            rLayout.addWidget(rLabelName)
            rLayout.addStretch(1)
            rLayout.addWidget(rLabelValue)
            rLayout.addStretch(1)
        else:
            rLayout.addWidget(rLabelName)
            rLayout.addWidget(rLabelValue)
            rLayout.addWidget(rLabelIncome)
        self.resourceList.addWidget(rWidget)
            
    def updateLabels(self):
        state = self.gameUI.state
        for rState in state.resources.values():
            rLabelValue = self.rValueLabels[rState.info.name]
            if rState.info.name == "Processors":
                rLabelValue.setText(f"{rState.storage} ({state.freeProcessorCount} unassigned)")
            else:
                rLabelIncome = self.rIncomeLabels[rState.info.name]

                c = rState.count
                if c >= 100000:
                    cStr = f"{c:.3e}"
                else:
                    cStr = f"{c:.1f}".rstrip('0').rstrip('.')

                rLabelValue.setText(f"{cStr} / {round(rState.storage)}")
                rLabelIncome.setText(f"{rState.income * state.database.params.intervalsPerSecond} /s")
