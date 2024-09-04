
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

from game.util.styleSheets import StyleSheets

class ResearchButtonWidget(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, gameUI : GameUI, rName : str):
        super().__init__()
        self.rName = rName
        self.gameUI = gameUI
        
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        state = gameUI.state
        rState = state.research[rName]
        rInfo : BuildingInfo = rState.info
        rCost = state.getResearchCost(rName)
        
        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Name and count
        titleWidget = self.makeTitleWidget()
        
        # Icon and resource (IR) list
        
        researchIconSize = 86
        researchIconLabel = gameUI.makeIconLabel('icons/research/' + rName + '.png', researchIconSize, researchIconSize)
        researchIconLabel.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
        
        rListWidget = QWidget()
        rListLayout = QVBoxLayout(rListWidget)
        rListLayout.setSpacing(0)
        rListLayout.setContentsMargins(0, 0, 0, 0)
        #rListLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        
        self.resourceNameLabels = {}
        self.resourceValueLabels = {}
        
        for resourceName, resourceCost in rCost.r.items():
            rWidget = self.makeResourceRowWidget(resourceName, resourceCost, True)
            rListLayout.addWidget(rWidget)
        rListLayout.addStretch()

        IRWidget = QWidget()
        IRLayout = QGridLayout(IRWidget)
        IRLayout.setContentsMargins(0, 0, 0, 0)
        
        IRLayout.addWidget(researchIconLabel, 0, 0, Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter)
        
        #rListWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)
        IRLayout.addWidget(rListWidget, 0, 1)
        IRLayout.setColumnStretch(0, 0)
        IRLayout.setColumnStretch(1, 1)
        IRWidget.setMinimumHeight(researchIconSize)
        
        descWidget = QLabel(rInfo.description)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        layout.addWidget(titleWidget)
        layout.addWidget(IRWidget)
        layout.addWidget(descWidget)
        
        layout.addStretch(0)
        
        self.updateLabels()
        
    def makeResourceRowWidget(self, resourceName : str, resourceValue : float, addToDicts : bool):
        rWidget = QWidget()
        rLayout = QGridLayout(rWidget)
        rLayout.setSpacing(0)
        rLayout.setContentsMargins(0, 0, 0, 0)
        rLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
            
        resourceIconLabel = self.gameUI.makeIconLabel('icons/resources/' + resourceName + '.png', 20, 20)
        resourceIconLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        
        resourceNameLabel  = QLabel(f"{resourceName}")
        
        valueText = f"{resourceValue}"
        resourceValueLabel = QLabel(valueText)
        resourceNameLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        resourceValueLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)
        resourceNameLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
        resourceValueLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)

        if addToDicts:
            self.resourceNameLabels[resourceName] = resourceNameLabel
            self.resourceValueLabels[resourceName] = resourceValueLabel
        
        rLayout.addWidget(resourceIconLabel, 0, 0)
        rLayout.addWidget(resourceNameLabel, 0, 1)
        rLayout.addWidget(resourceValueLabel, 0, 2)
        rLayout.setColumnStretch(0, 0)
        rLayout.setColumnStretch(1, 3)
        rLayout.setColumnStretch(2, 3)
        return rWidget
        
    def makeTitleWidget(self):
        nameLabel = QLabel(f"{self.rName}")
        #self.countLabel = QLabel("")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        #self.countLabel.setAlignment(Qt.AlignmentFlag.AlignRight)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        #self.countLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        titleWidget = QWidget()
        titleLayout = QHBoxLayout(titleWidget)
        titleLayout.setContentsMargins(0, 0, 0, 0)
        titleLayout.addWidget(nameLabel)
        #titleLayout.addWidget(self.countLabel)
        return titleWidget
        
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        researchCost = self.gameUI.state.getResearchCost(self.rName)
        canAfford = self.gameUI.state.canAffordCost(researchCost)
        
        if canAfford:
            if self.underMouse():
                painter.setBrush(QColor(150, 200, 150, 100))
            else:
                painter.setBrush(QColor(180, 230, 180, 100))
        else:
            painter.setBrush(QColor(250, 200, 200, 100))

        painter.setPen(QColor(180, 180, 180))
        painter.drawRoundedRect(self.rect(), 10, 10)

    def enterEvent(self, event):
        self.update()

    def leaveEvent(self, event):
        self.update()
        
    def updateLabels(self):
        state : GameState = self.gameUI.state
        rState : ResearchState = state.research[self.rName]
        researchCost : ResourceList = state.getResearchCost(self.rName)
        
        for resourceName, v in researchCost.r.items():
            rValue = state.resources[resourceName].count
            if rValue < v:
                self.resourceNameLabels[resourceName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST_RED)
                self.resourceValueLabels[resourceName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST_RED)
            else:
                self.resourceNameLabels[resourceName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
                self.resourceValueLabels[resourceName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
        
        self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.rName)
            

class ResearchCollapsibleSection(QWidget):
    def __init__(self, gameUI : GameUI, title : str):
        super().__init__()
        self.gameUI = gameUI
        self.title = title
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout()
        self.setLayout(layout)
        
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(2)

        # Create a button for expanding/collapsing with an arrow icon
        
        self.toggleButton = QPushButton(self.title)
        self.toggleButton.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)
        self.toggleButton.setCheckable(True)
        self.toggleButton.setChecked(True)
        self.toggleButton.clicked.connect(self.toggleContent)
        self.updateArrow()
        layout.addWidget(self.toggleButton)

        # Create a scroll area for the content
        self.scrollArea = QScrollArea()
        self.scrollArea.setWidgetResizable(True)
        self.scrollArea.setFrameShape(QFrame.Shape.NoFrame)
        layout.addWidget(self.scrollArea)

        # Create a widget to hold the content
        self.contentWidget = QWidget()
        self.contentLayout = QGridLayout(self.contentWidget)
        
        self.researchWidgets : Dict[str, ResearchButtonWidget] = {}
        
        index = 0
        for rInfo in self.gameUI.database.research.values():
            if rInfo.category != self.title:
                continue
            
            rWidget = ResearchButtonWidget(self.gameUI, rInfo.name)

            row = index // 3
            col = index % 3
            self.contentLayout.addWidget(rWidget, row, col)
            self.researchWidgets[rInfo.name] = rWidget
            
            rWidget.clicked.connect(self.gameUI.purchaseResearch)
            
            index += 1

        self.scrollArea.setWidget(self.contentWidget)

    def toggleContent(self):
        self.scrollArea.setVisible(self.toggleButton.isChecked())
        self.updateArrow()

    def updateArrow(self):
        arrowText = ' \u25C0'
        if self.toggleButton.isChecked():
            arrowText = ' \u25BC'
        self.toggleButton.setText(self.title + arrowText)

    def addWidget(self, widget):
        self.contentLayout.addWidget(widget)

class ResearchViewWidget(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        
        mainLayout = QVBoxLayout()
        self.setLayout(mainLayout)

        mainLayout.setContentsMargins(2, 2, 2, 2)
        mainLayout.setSpacing(2)
        
        # Create collapsible sections
        
        self.sections = []
        for rCategory in self.gameUI.state.database.params.researchCategories:
            section = ResearchCollapsibleSection(gameUI, rCategory)
            section.setMinimumWidth(845)

            self.sections.append(section)
            mainLayout.addWidget(section)
        
    def updateLabels(self):
        pass
        for widget in self.buildingWidgets.values():
            widget.updateLabels()
        
    