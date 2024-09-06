
from __future__ import annotations

import random
from functools import partial

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import GameDatabase
from game.core.gameState import GameState
from game.ui.collapsibleMenuWidget import CollapsibleMenuWidget, CollapsibleSectionEntries
from game.util.enums import GameWindowMode
from game.util.styleSheets import StyleSheets

class CommandButtonWidget(QPushButton):
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
        addButton.clicked.connect(partial(gameUI.addCommandToProgram, name))
        
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
        
        self.rNameLabels = {}
        self.rCostLabels = {}
        
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
        rCostLabel = QLabel(f"{value:+}")
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

        self.rNameLabels[rName] = rNameLabel
        self.rCostLabels[rName] = rCostLabel

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
        
    def updateLabels(self):
        
        state : GameState = self.gameUI.state
        commandCost : ResourceList = self.state.getCommandCost(self.name)
        for rName, v in commandCost.r.items():
            rValue = state.resources[rName].count
            if rValue < v:
                self.rNameLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST_RED)
                self.rCostLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST_RED)
            else:
                self.rNameLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
                self.rCostLabels[rName].setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
        self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.name)
            
"""class CommandViewWidget(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        
        commandGridLayout = QGridLayout(self)
        
        self.commandWidgets : Dict[str, CommandButtonWidget] = {}
        
        for index, cName in enumerate(gameUI.database.commands.keys()):
            cWidget = CommandButtonWidget(gameUI, cName)

            row = index // 3
            col = index % 3
            commandGridLayout.addWidget(cWidget, row, col)
            self.commandWidgets[cName] = cWidget
            
            cWidget.clicked.connect(gameUI.runCommand)
            
    def updateLabels(self):
        for widget in self.commandWidgets.values():
            widget.updateLabels()"""
    
class CommandView():
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        
        self.sections: Dict[str, CollapsibleSectionEntries] = {}

        for commandCategory in self.gameUI.state.database.params.commandCategories:
            self.sections[commandCategory] = CollapsibleSectionEntries(commandCategory)
        
        for cState in self.gameUI.state.commands.values():
            entryWidget = CommandButtonWidget(self.gameUI, cState.info.name)
            entryWidget.clicked.connect(gameUI.runCommand)
            self.sections[cState.info.category].childWidgets.append(entryWidget)

        self.mainWidget = CollapsibleMenuWidget(gameUI, self.sections, "grid")
        
    def updateLabels(self):
        self.mainWidget.updateLabels()
