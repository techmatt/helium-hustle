
from __future__ import annotations

import random

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import GameDatabase
from gameState import GameState

from enums import GameWindowMode
from iconGrid import IconGrid
from resourceDisplay import ResourceDisplay
from styleSheets import StyleSheets

class CommandWidget(QWidget):
    def __init__(self, gameUI : GameUI, name : str):
        super().__init__()
        
        self.name = name
    
        layout = QHBoxLayout()
        
        cState = gameUI.state.commands[name]
        
        nameButton = QPushButton(f"{name}")
        nameButton.setStyleSheet(StyleSheets.MODERN_BUTTON)
        nameButton.clicked.connect(lambda: gameUI.runCommand(name))

        layout.addWidget(nameButton)

        self.setLayout(layout)

class BuildingButton(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, state : GameState, name : str):
        super().__init__()
        self.name = name
        self.state = state
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        bState = state.buildings[name]
        bInfo = bState.info
        buildingCost = state.getBuildingCost(name)

        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Name and count
        nameLabel = QLabel(f"{name}")
        countLabel = QLabel(f"({bState.count})")
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
        buildingIconLabel = QLabel()
        buildingIconPath = 'icons/buildings/' + name + '.png'
        pixmap = QPixmap(buildingIconPath).scaled(
            buildingIconSize, buildingIconSize, 
            Qt.AspectRatioMode.KeepAspectRatio, 
            Qt.TransformationMode.SmoothTransformation
        )
        buildingIconLabel.setPixmap(pixmap)
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
            
            iconPath = 'icons/resources/' + rName + '.png'
            rIconLabel = QLabel()
            pixmap = QPixmap(iconPath).scaled(
                20, 20,
                Qt.AspectRatioMode.KeepAspectRatio, 
                Qt.TransformationMode.SmoothTransformation
            )
            rIconLabel.setPixmap(pixmap)
            rIconLabel.setFixedSize(20, 20)
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
        