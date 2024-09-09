
from __future__ import annotations

import math
import random
from functools import partial

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton,
    QLabel, QGridLayout, QSizePolicy, QScrollArea, QFrame, QProgressBar )
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import BuildingInfo, GameDatabase
from game.core.gameState import BuildingState, GameState
from game.ui.collapsibleMenuWidget import CollapsibleMenuWidget, CollapsibleSectionEntries

from game.util.styleSheets import StyleSheets

class IdeologyProgressBar(QProgressBar):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setTextVisible(True)
        self.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)

    def setValue(self, value):
        super().setValue(math.floor(value))
        self.updateText()

    def setMaximum(self, maximum):
        super().setMaximum(math.floor(maximum))
        self.updateText()

    def updateText(self):
        current = self.value()
        maximum = self.maximum()
        self.setFormat(f"{current} / {maximum}")

    def setValueAndMaximum(self, value, maximum):
        self.setMaximum(maximum)
        self.setValue(value)
        
class IdeologyWidget(QWidget):
    def __init__(self, gameUI : GameUI, iName : str):
        super().__init__()
        self.iName = iName
        self.gameUI = gameUI
        
        #self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        state = gameUI.state
        iState = state.ideologies[iName]
        iInfo : IdeologyInfo = iState.info
        
        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        titleWidget = self.makeTitleWidget()
        
        # icon + (progress bar / rank) widget
        IBWidget = QWidget()
        IBLayout = QHBoxLayout(IBWidget)

        ideologyIconW, ideologyIconH = 233, 80
        ideologyIconLabel = gameUI.makeIconLabel('icons/ideologiesLarge/' + iName + '.png', ideologyIconW, ideologyIconH)
        ideologyIconLabel.setAlignment(Qt.AlignmentFlag.AlignHCenter | Qt.AlignmentFlag.AlignTop)
        IBLayout.addWidget(ideologyIconLabel)
        
        # progress bar / rank widget
        BRWidget = QWidget()
        BRLayout = QVBoxLayout(BRWidget)
        self.progressBar = IdeologyProgressBar()
        self.rankLabel = QLabel(f"Rank: X")
        self.rankLabel.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.rankLabel.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)
        BRLayout.addWidget(self.progressBar)
        BRLayout.addWidget(self.rankLabel)
        IBLayout.addWidget(BRWidget)
        
        """rListWidget = QWidget()
        rListLayout = QVBoxLayout(rListWidget)
        rListLayout.setSpacing(0)
        rListLayout.setContentsMargins(0, 0, 0, 0)
        #rListLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        
        self.resourceNameLabels = {}
        self.resourceValueLabels = {}
        
        for resourceName, resourceCost in rCost.r.items():
            rWidget = self.makeResourceRowWidget(resourceName, resourceCost, True)
            rListLayout.addWidget(rWidget)
        rListLayout.addStretch()"""

        descWidget = QLabel(iInfo.flavorText)
        descWidget.setAlignment(Qt.AlignmentFlag.AlignHCenter | Qt.AlignmentFlag.AlignTop)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        layout.addWidget(titleWidget)
        layout.addWidget(IBWidget)
        layout.addWidget(descWidget)
        
        #layout.addStretch(0)
        
        self.updateLabels()
        
    def makeTitleWidget(self):
        nameLabel = QLabel(f"{self.iName}")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignHCenter)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        titleWidget = QWidget()
        titleLayout = QHBoxLayout(titleWidget)
        titleLayout.setContentsMargins(0, 0, 0, 0)
        titleLayout.addWidget(nameLabel)
        return titleWidget
        
    def paintEvent(self, event):
        state : GameState = self.gameUI.state
        iState = state.ideologies[self.iName]
    
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        if iState.totalScore == 0:
            painter.setBrush(QColor(255, 255, 200, 100))
        elif iState.totalScore > 0:
            painter.setBrush(QColor(180, 230, 180, 100))
        else:
            painter.setBrush(QColor(255, 150, 150, 100))
        
        painter.setPen(QColor(180, 180, 180))
        painter.drawRoundedRect(self.rect(), 10, 10)

    def enterEvent(self, event):
        self.update()

    def leaveEvent(self, event):
        self.update()
        
    def updateLabels(self):
        state : GameState = self.gameUI.state
        iState = state.ideologies[self.iName]
        self.progressBar.setValueAndMaximum(iState.localRankScore, iState.localRankThreshold)
        
        if iState.totalScore > 0.0:
            self.progressBar.setStyleSheet(StyleSheets.PROGRESS_BAR_GREEN)
        else:
            self.progressBar.setStyleSheet(StyleSheets.PROGRESS_BAR_RED)
            
        self.rankLabel.setText(f"Rank: {iState.rank}")
        self.update()

class IdeologyView():
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        state = gameUI.state
        
        self.mainWidget = QWidget()
        
        mainLayout = QVBoxLayout(self.mainWidget)
        self.mainWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        
        mainLayout.setContentsMargins(2, 2, 2, 2)
        mainLayout.setSpacing(2)
        
        # Create a scroll area for the content
        self.scrollArea = QScrollArea()
        self.scrollArea.setWidgetResizable(True)
        self.scrollArea.setFrameShape(QFrame.Shape.NoFrame)
        self.scrollArea.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        mainLayout.addWidget(self.scrollArea)

        # Create collapsible sections
        
        self.contentWidget = QWidget()
        self.contentLayout = QVBoxLayout(self.contentWidget)
        self.contentWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        
        self.ideologyWidgets : List[IdeologyWidget] = []
        for iState in state.ideologies.values():
            ideologyWidget = IdeologyWidget(gameUI, iState.info.name)
            ideologyWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)

            self.ideologyWidgets.append(ideologyWidget)
            self.contentLayout.addWidget(ideologyWidget)
            
        self.contentLayout.addStretch(1)
        self.scrollArea.setWidget(self.contentWidget)
        
    def updateLabels(self):
        for w in self.ideologyWidgets:
            w.updateLabels()
