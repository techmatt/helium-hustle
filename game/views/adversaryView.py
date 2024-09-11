
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

        iconStatsWidget = QWidget()
        iconStatsLayout = QHBoxLayout(iconStatsWidget)
        iconStatsLayout.setContentsMargins(2, 2, 2, 2)
        iconWidget = gameUI.makeIconLabel('icons/armies/' + name + '.png', 60, 60)
        iconWidget.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        
        statsAWidget = QWidget()
        statsALayout = QVBoxLayout(statsAWidget)
        
        self.strengthLabel = QLabel(f"Enemy forces: X")
        self.strengthLabel.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)
        self.effectivenessLabel = QLabel(f"Effectiveness: X%")
        self.effectivenessLabel.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)
        statsALayout.addWidget(self.strengthLabel)
        statsALayout.addWidget(self.effectivenessLabel)
        
        iconStatsLayout.addWidget(iconWidget)
        iconStatsLayout.addWidget(statsAWidget)
        
        
        # Resource list
        
        attrListWidget = QWidget()
        attrListLayout = QVBoxLayout(attrListWidget)
        attrListLayout.setSpacing(0)
        attrListLayout.setContentsMargins(0, 0, 0, 0)
        attrListLayout.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
        
        self.spawnRateLabel = QLabel(f"Spawn rate: X/s")
        self.nSurgeTimeLabel = QLabel(f"Time to next surge: X years")
        self.nSurgeStrLabel = QLabel(f"Next surge force count: X")
        
        self.spawnRateLabel.setStyleSheet(StyleSheets.GENERAL_12PT)
        self.nSurgeTimeLabel.setStyleSheet(StyleSheets.GENERAL_12PT)
        self.nSurgeStrLabel.setStyleSheet(StyleSheets.GENERAL_12PT)
        
        attrListLayout.addWidget(self.spawnRateLabel)
        attrListLayout.addWidget(self.nSurgeTimeLabel)
        attrListLayout.addWidget(self.nSurgeStrLabel)
        
        attrListLayout.addStretch()

        descWidget = QLabel(aInfo.flavorText)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        layout.addWidget(titleWidget)
        layout.addWidget(iconStatsWidget)
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
        state = self.gameUI.state
        aState = state.adversaries[self.name]
        self.update()
    
class AdversaryView():
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        state = gameUI.state
        
        self.sections: Dict[str, CollapsibleSectionEntries] = {}

        for adversaryCategory in state.database.params.adversaryCategories:
            self.sections[adversaryCategory] = CollapsibleSectionEntries(adversaryCategory)
        
        for aState in state.adversaries.values():
            entryWidget = AdversaryButtonWidget(self.gameUI, aState.info.name)
            self.sections[aState.info.category].childWidgets.append(entryWidget)

        self.mainWidget = CollapsibleMenuWidget(gameUI, self.sections, "grid")
        
    def updateLabels(self):
        self.mainWidget.updateLabels()
