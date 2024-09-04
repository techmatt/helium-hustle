
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

class CollapsibleSectionEntries:
    def __init__(self, name : str):
        self.name = name
        self.entries : List[QWidget] = []
        
class CollapsibleSectionWidget(QWidget):
    def __init__(self, gameUI : GameUI, title : str, entries : CollapsibleSectionEntries):
        super().__init__()
        self.gameUI = gameUI
        self.entries = entries
        self.title = title
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout(self)
        #self.setLayout(layout)
        
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
        
        index = 0
        for entry in self.entries:
            row = index // 3
            col = index % 3
            self.contentLayout.addWidget(entry, row, col)
            
            entry.clicked.connect(self.gameUI.purchaseResearch)
            
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

class CollapsibleMenuWidget(QWidget):
    def __init__(self, gameUI : GameUI, sections : List[CollapsibleSectionEntries]):
        super().__init__()
        self.gameUI = gameUI
        
        mainLayout = QVBoxLayout()
        self.setLayout(mainLayout)

        mainLayout.setContentsMargins(2, 2, 2, 2)
        mainLayout.setSpacing(2)
        
        # Create collapsible sections
        
        self.sections : List[CollapsibleSectionWidget] = []
        for section in sections:
            sectionWidget = CollapsibleSectionWidget(gameUI, section)
            section.setMinimumWidth(845)

            self.sections.append(sectionWidget)
            mainLayout.addWidget(sectionWidget)
        
    def updateLabels(self):
        for widget in self.buildingWidgets.values():
            widget.updateLabels()
        
    