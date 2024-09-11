
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

class AdversaryButtonWidget(QPushButton):
    def __init__(self, gameUI : GameUI, name : str):
        super().__init__()
        self.name = name
        self.gameUI = gameUI
        self.state = gameUI.state
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        aState = self.state.adversaries[name]
        aInfo = aState.info
        
        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Name and count
        nameLabel = QLabel(f"{name}")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        titleWidget = QWidget()
        titleLayout = QHBoxLayout(titleWidget)
        titleLayout.setContentsMargins(0, 0, 0, 0)
        titleLayout.addWidget(nameLabel)
        titleLayout.addStretch(1)
        titleLayout.addWidget(addButton)
        
        # Resource list
        
        attrListWidget = QWidget()
        attrListLayout = QVBoxLayout(rListWidget)
        attrListLayout.setSpacing(0)
        attrListLayout.setContentsMargins(0, 0, 0, 0)
        attrListLayout.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
        
        self.strengthLabel = QLabel(f"Strength: {aState.strength}")
        self.effectivenessLabel = QLabel(f"Effectiveness: {aState.effectiveness}")
        self.spawnRateLabel = QLabel(f"Spawn rate: {self.state.convertPerTickToPerSecond(aState.spawnRate)} /s")
        self.nSurgeTimeLabel = QLabel(f"Time to next surge: {self.state.convertTicksToYears(aState.ticksToSurge)} years")
        self.nSurgeStrLabel = QLabel(f"Next surge strength: {aState.nextSurgeStrength}")
        
        attrListLayout.addWidget(self.strengthLabel)
        attrListLayout.addWidget(self.effectivenessLabel)
        attrListLayout.addWidget(self.spawnRateLabel)
        attrListLayout.addWidget(self.nSurgeTimeLabel)
        attrListLayout.addWidget(self.nSurgeStrLabel)
        
        attrListLayout.addStretch()

        descWidget = QLabel(aInfo.description)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        layout.addWidget(titleWidget)
        layout.addWidget(attrListWidget)
        layout.addWidget(descWidget)
        
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
        self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.name)
            
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
