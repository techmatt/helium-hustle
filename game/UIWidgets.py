
from __future__ import annotations

import random
from functools import partial

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import GameDatabase
from gameState import GameState

from enums import GameWindowMode
from iconGrid import IconGrid
from resourceDisplay import ResourceDisplay
from styleSheets import StyleSheets

class ProgramUIElements():
    def __init__(self, gameUI : GameUI):
        self.gameUI = gameUI
        
        self.programLabel = QLabel("Programs")
        self.programLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        self.programSelectWidget = QWidget()
        programSelectLayout = QHBoxLayout(self.programSelectWidget)
        self.programIndexButtons = []
        for i in range(0, len(gameUI.state.programs)):
            button = QPushButton(str(i+1))
            #button.setStyleSheet(StyleSheets.BUILDING_TITLE)
            button.clicked.connect(partial(gameUI.changeVisibleProgramIndex, i))
            programSelectLayout.addWidget(button)
            self.programIndexButtons.append(button)

        self.processorAllocationWidget = QWidget()
        processorAllocationLayout = QHBoxLayout(self.processorAllocationWidget)
        processorIcon = gameUI.makeIconLabel('icons/resources/processors.png', 32, 32)
        
        activeProgram = gameUI.state.programs[gameUI.visibleProgramIndex]
        self.assignedProcessorsLabel = QLabel("")
        
        processorSubButton = QPushButton("-")
        processorAddButton = QPushButton("+")

        buttonSize = QSize(25, 25)
        processorSubButton.setFixedSize(buttonSize)
        processorAddButton.setFixedSize(buttonSize)
        
        self.assignedProcessorsLabel.setStyleSheet(StyleSheets.RESOURCE_LIST_TEXT)
        processorSubButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        processorAddButton.setStyleSheet(StyleSheets.BUILDING_TITLE)

        processorAllocationLayout.addWidget(processorIcon)
        processorAllocationLayout.addWidget(self.assignedProcessorsLabel)
        processorAllocationLayout.addStretch(1)
        processorAllocationLayout.addWidget(processorSubButton)
        processorAllocationLayout.addWidget(processorAddButton)
        
        self.updateVisisbleProgramIndex()

    def updateVisisbleProgramIndex(self):
        state : GameState = self.gameUI.state
        for i in range(0, len(state.programs)):
            if i == self.gameUI.visibleProgramIndex:
                self.programIndexButtons[i].setStyleSheet(StyleSheets.SELECTED_BUTTON)
            else:
                self.programIndexButtons[i].setStyleSheet(StyleSheets.BUILDING_TITLE)

        activeProgram = state.programs[self.gameUI.visibleProgramIndex]
        self.assignedProcessorsLabel.setText(f"{activeProgram.assignedProcessors} processors assigned ({state.freeProcessorCount} free)")

class CommandButton(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, gameUI : GameUI, name : str):
        super().__init__()
        self.name = name
        self.gameUI = gameUI
        self.state = gameUI.state
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        cState = self.state.commands[name]
        cInfo = cState.info
        commandCost = self.state.getCommandCost(name)

        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Name and count
        nameLabel = QLabel(f"{name}")
        addButton = QPushButton("+")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        addButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        titleWidget = QWidget()
        titleLayout = QHBoxLayout(titleWidget)
        titleLayout.setContentsMargins(0, 0, 0, 0)
        titleLayout.addWidget(nameLabel)
        titleLayout.addStretch(1)
        titleLayout.addWidget(addButton)
        
        # Resource list
        
        rListWidget = QWidget()
        rListLayout = QVBoxLayout(rListWidget)
        rListLayout.setSpacing(0)
        rListLayout.setContentsMargins(0, 0, 0, 0)
        rListLayout.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
        
        for rName, production in cInfo.production.items():
            rWidget = self.makeResourceWidget(gameUI, rName, production)
            rListLayout.addWidget(rWidget)
            
        for rName, cost in cInfo.cost.items():
            rWidget = self.makeResourceWidget(gameUI, rName, -cost)
            rListLayout.addWidget(rWidget)
        rListLayout.addStretch()

        descWidget = QLabel(cInfo.description)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        layout.addWidget(titleWidget)
        layout.addWidget(rListWidget)
        layout.addWidget(descWidget)
        
    def makeResourceWidget(self, gameUI, rName, value):
        rWidget = QWidget()
        rLayout = QGridLayout(rWidget)
        rLayout.setSpacing(0)
        rLayout.setContentsMargins(0, 0, 0, 0)
        rLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
            
        iconPath = 'icons/resources/' + rName + '.png'
        rIconLabel = QLabel()
        rIconLabel.setPixmap(gameUI.pixmapCache.getPixmap(iconPath, 20, 20))
        rIconLabel.setFixedSize(20, 20)
        rIconLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
            
        rNameLabel = QLabel(f"{rName}")
        rCostLabel = QLabel(f"{value}")
        rNameLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        rCostLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)
        rNameLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
        rCostLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
            
        rLayout.addWidget(rIconLabel, 0, 0)
        rLayout.addWidget(rNameLabel, 0, 1)
        rLayout.addWidget(rCostLabel, 0, 2)
        rLayout.setColumnStretch(0, 0)
        rLayout.setColumnStretch(1, 3)
        rLayout.setColumnStretch(2, 3)
        return rWidget
    
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        commandCost = self.state.getCommandCost(self.name)
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

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.name)
            
class BuildingButton(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, gameUI : GameUI, name : str):
        super().__init__()
        self.name = name
        self.gameUI = gameUI
        self.state = gameUI.state
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        bState = self.state.buildings[name]
        bInfo = bState.info
        buildingCost = self.state.getBuildingCost(name)

        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Name and count
        nameLabel = QLabel(f"{name}")
        countLabel = QLabel(f"({bState.totalCount})")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        countLabel.setAlignment(Qt.AlignmentFlag.AlignRight)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        countLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        titleWidget = QWidget()
        titleLayout = QHBoxLayout(titleWidget)
        titleLayout.setContentsMargins(0, 0, 0, 0)
        titleLayout.addWidget(nameLabel)
        titleLayout.addWidget(countLabel)
        
        # Icon and resource (IR) list
        
        buildingIconSize = 86
        buildingIconLabel = gameUI.makeIconLabel('icons/buildings/' + name + '.png', buildingIconSize, buildingIconSize)
        buildingIconLabel.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
        
        rListWidget = QWidget()
        rListLayout = QVBoxLayout(rListWidget)
        rListLayout.setSpacing(0)
        rListLayout.setContentsMargins(0, 0, 0, 0)
        #rListLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        
        for rName, cost in buildingCost.costs.items():
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
        
        layout.addWidget(titleWidget)
        layout.addWidget(IRWidget)
        layout.addWidget(descWidget)
        
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        buildingCost = self.state.getBuildingCost(self.name)
        canAfford = self.state.canAffordCost(buildingCost)
        
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

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.name)

#rWidget = QLabel("text!")
#color = QColor(*random.sample(range(255), 3))
#rListWidget.setStyleSheet("background-color: {}".format(color.name()))
        