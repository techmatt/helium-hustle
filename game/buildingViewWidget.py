
from __future__ import annotations

import random
from functools import partial

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import BuildingInfo, GameDatabase
from gameState import GameState

from enums import GameWindowMode
from iconGrid import IconGrid
from resourceDisplay import ResourceDisplay
from styleSheets import StyleSheets

class BuildingButtonWidget(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, gameUI : GameUI, bName : str):
        super().__init__()
        self.bName = bName
        self.gameUI = gameUI
        self.state = gameUI.state
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        bState = self.state.buildings[bName]
        bInfo : BuildingInfo = bState.info
        buildingCost = self.state.getBuildingCost(bName)

        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # disable buttons (optional) and destroy button
        buttonSize = QSize(25, 25)

        editButtonsWidget = QWidget()
        editButtonsLayout = QHBoxLayout(editButtonsWidget)
        editButtonsLayout.setContentsMargins(0, 0, 0, 0)
        editButtonsLayout.addStretch(1)
        
        self.removeButton = QPushButton("x")
        self.removeButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        self.removeButton.setFixedSize(buttonSize)
        
        if bInfo.canDeactivate:
            self.subButton = QPushButton("-")
            self.addButton = QPushButton("+")
        
            self.subButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
            self.addButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
            self.subButton.setFixedSize(buttonSize)
            self.addButton.setFixedSize(buttonSize)

            self.subButton.clicked.connect(partial(gameUI.modifyBuildingActive, bName, -1))
            self.addButton.clicked.connect(partial(gameUI.modifyBuildingActive, bName, 1))
            
            editButtonsLayout.addWidget(self.subButton)
            editButtonsLayout.addWidget(self.addButton)
            
        self.removeButton.clicked.connect(partial(gameUI.removeBuilding, bName))
        editButtonsLayout.addWidget(self.removeButton)

        # Name and count
        nameLabel = QLabel(f"{bName}")
        self.countLabel = QLabel("")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        self.countLabel.setAlignment(Qt.AlignmentFlag.AlignRight)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        self.countLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        titleWidget = QWidget()
        titleLayout = QHBoxLayout(titleWidget)
        titleLayout.setContentsMargins(0, 0, 0, 0)
        titleLayout.addWidget(nameLabel)
        titleLayout.addWidget(self.countLabel)
        
        # Icon and resource (IR) list
        
        buildingIconSize = 86
        buildingIconLabel = gameUI.makeIconLabel('icons/buildings/' + bName + '.png', buildingIconSize, buildingIconSize)
        buildingIconLabel.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
        
        rListWidget = QWidget()
        rListLayout = QVBoxLayout(rListWidget)
        rListLayout.setSpacing(0)
        rListLayout.setContentsMargins(0, 0, 0, 0)
        #rListLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        
        self.rNameLabels = {}
        self.rCostLabels = {}
        
        for rName, cost in buildingCost.r.items():
            # TODO: refactor into self.makeResourceWidget
            rWidget = QWidget()
            rLayout = QGridLayout(rWidget)
            rLayout.setSpacing(0)
            rLayout.setContentsMargins(0, 0, 0, 0)
            rLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
            
            rIconLabel = gameUI.makeIconLabel('icons/resources/' + rName + '.png', 20, 20)
            rIconLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
            
            rNameLabel = QLabel(f"{rName}")
            rCostLabel = QLabel(f"{cost}")
            rNameLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
            rCostLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)
            rNameLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
            rCostLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)

            self.rNameLabels[rName] = rNameLabel
            self.rCostLabels[rName] = rCostLabel
            
            rLayout.addWidget(rIconLabel, 0, 0)
            rLayout.addWidget(rNameLabel, 0, 1)
            rLayout.addWidget(rCostLabel, 0, 2)
            rLayout.setColumnStretch(0, 0)
            rLayout.setColumnStretch(1, 3)
            rLayout.setColumnStretch(2, 3)
        
            rListLayout.addWidget(rWidget)
        rListLayout.addStretch()

        IRWidget = QWidget()
        IRLayout = QGridLayout(IRWidget)
        IRLayout.setContentsMargins(0, 0, 0, 0)
        
        IRLayout.addWidget(buildingIconLabel, 0, 0, Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter)
        
        #rListWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)
        IRLayout.addWidget(rListWidget, 0, 1)
        IRLayout.setColumnStretch(0, 0)
        IRLayout.setColumnStretch(1, 1)
        IRWidget.setMinimumHeight(buildingIconSize)
        
        descWidget = QLabel(bInfo.description)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        layout.addWidget(editButtonsWidget)
        layout.addWidget(titleWidget)
        layout.addWidget(IRWidget)
        layout.addWidget(descWidget)
        
        self.updateLabels()
        
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        commandCost = self.state.getBuildingCost(self.bName)
        canAfford = self.state.canAffordCost(commandCost)
        
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
        bState : BuildingState = state.buildings[self.bName]
        commandCost : ResourceList = self.state.getBuildingCost(self.bName)
        
        for rName, v in commandCost.r.items():
            rValue = state.resources[rName].count
            if rValue < v:
                self.rNameLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST_RED)
                self.rCostLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST_RED)
            else:
                self.rNameLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
                self.rCostLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
                
        if bState.info.canDeactivate:
            self.countLabel.setText(f"({bState.activeCount}/{bState.totalCount})")
        else:
            self.countLabel.setText(f"({bState.totalCount})")
        
        self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.bName)
            
class BuildingViewWidget(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        
        buildingGridLayout = QGridLayout(self)
        
        self.buildingWidgets : Dict[str, BuildingButtonWidget] = {}
        
        for index, bName in enumerate(gameUI.database.buildings.keys()):
            bWidget = BuildingButtonWidget(gameUI, bName)

            row = index // 3
            col = index % 3
            buildingGridLayout.addWidget(bWidget, row, col)
            self.buildingWidgets[bName] = bWidget
            
            bWidget.clicked.connect(gameUI.buildBuilding)
        
    def updateLabels(self):
        for widget in self.buildingWidgets.values():
            widget.updateLabels()
        
    