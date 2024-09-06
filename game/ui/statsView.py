
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

class StatsView():
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        
        self.sections: Dict[str, CollapsibleSectionEntries] = {}
        
        statCategories = ["Research", "Events"]

        for statCategory in statCategories:
            self.sections[statCategory] = CollapsibleSectionEntries(statCategory)
        
        for rState in self.gameUI.state.research.values():
            rInfo = rState.info
            text = f"<b>{rInfo.name}</b>: {rInfo.description}"
            entryWidget = QLabel(text)
            entryWidget.setWordWrap(True)
            entryWidget.setStyleSheet(StyleSheets.GENERAL_12PT)
            self.sections["Research"].childWidgets.append(entryWidget)
            
        for eState in self.gameUI.state.events.values():
            if not eState.ongoing:
                continue
            eInfo = eState.info
            text = f"<b>{eInfo.name}</b>: {eInfo.mechanicsText}"
            entryWidget = QLabel(text)
            entryWidget.setWordWrap(True)
            entryWidget.setStyleSheet(StyleSheets.GENERAL_12PT)
            self.sections["Events"].childWidgets.append(entryWidget)

        self.mainWidget = CollapsibleMenuWidget(gameUI, self.sections, "list")
        
    def updateLabels(self):
        self.mainWidget.updateLabels()
