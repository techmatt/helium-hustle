
from __future__ import annotations

import random
from functools import partial

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton,
    QLabel, QGridLayout, QSizePolicy, QScrollArea, QFrame )
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import BuildingInfo, GameDatabase
from game.core.gameState import BuildingState, GameState
from game.ui.collapsibleMenuWidget import CollapsibleMenuWidget, CollapsibleSectionEntries

from game.util.styleSheets import StyleSheets

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
        
        ideologyIconW, ideologyIconH = 233, 80
        ideologyIconLabel = gameUI.makeIconLabel('icons/ideologiesLarge/' + iName + '.png', ideologyIconW, ideologyIconH)
        ideologyIconLabel.setAlignment(Qt.AlignmentFlag.AlignHCenter | Qt.AlignmentFlag.AlignTop)
        
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
        layout.addWidget(ideologyIconLabel)
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
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        
        if self.underMouse():
            painter.setBrush(QColor(150, 200, 150, 100))
        else:
            painter.setBrush(QColor(180, 230, 180, 100))
        
        painter.setPen(QColor(180, 180, 180))
        painter.drawRoundedRect(self.rect(), 10, 10)

    def enterEvent(self, event):
        self.update()

    def leaveEvent(self, event):
        self.update()
        
    def updateLabels(self):
        state : GameState = self.gameUI.state
        
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
        pass
        #self.mainWidget.updateLabels()
