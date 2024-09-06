
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
    def __init__(self, title : str):
        self.title = title
        self.childWidgets : List[QWidget] = []
        
class CollapsibleSectionWidget(QWidget):
    def __init__(self, gameUI : GameUI, sectionEntry : CollapsibleSectionEntries):
        super().__init__()
        self.gameUI = gameUI
        self.childWidgets = sectionEntry.childWidgets
        self.title = sectionEntry.title
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout(self)
        
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

        # Create a widget to hold the content
        self.contentWidget = QWidget()
        self.contentWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        self.contentLayout = QGridLayout(self.contentWidget)
        
        index = 0
        for childWidget in self.childWidgets:
            row = index // 3
            col = index % 3
            self.contentLayout.addWidget(childWidget, row, col)
            
            #entry.clicked.connect(self.gameUI.purchaseResearch)
            
            index += 1
            
        layout.addWidget(self.contentWidget)

    def toggleContent(self):
        self.contentWidget.setVisible(self.toggleButton.isChecked())
        self.updateArrow()

    def updateArrow(self):
        arrowText = ' \u25C0'
        if self.toggleButton.isChecked():
            arrowText = ' \u25BC'
        self.toggleButton.setText(self.title + arrowText)

class CollapsibleMenuWidget(QWidget):
    def __init__(self, gameUI : GameUI, sectionEntries : Dict[str, CollapsibleSectionEntries]):
        super().__init__()
        self.gameUI = gameUI
        
        mainLayout = QVBoxLayout(self)
        self.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        
        mainLayout.setContentsMargins(2, 2, 2, 2)
        mainLayout.setSpacing(2)
        
        # Create a scroll area for the content
        self.scrollArea = QScrollArea()
        self.scrollArea.setWidgetResizable(True)
        self.scrollArea.setFrameShape(QFrame.Shape.NoFrame)
        self.scrollArea.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        #self.scrollArea.setMinimumWidth(600)
        #self.scrollArea.setMinimumHeight(600)
        #policy = self.scrollArea.sizePolicy()
        #policy.setVerticalStretch(1)
        #policy.setHorizontalStretch(1)
        #self.scrollArea.setSizePolicy(policy)


        mainLayout.addWidget(self.scrollArea)

        # Create collapsible sections
        
        self.contentWidget = QWidget()
        self.contentLayout = QVBoxLayout(self.contentWidget)
        self.contentWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        
        self.sectionWidgets : List[CollapsibleSectionWidget] = []
        for sectionEntry in sectionEntries.values():
            sectionWidget = CollapsibleSectionWidget(gameUI, sectionEntry)
            sectionWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
            #sectionWidget.setMinimumWidth(845)

            self.sectionWidgets.append(sectionWidget)
            self.contentLayout.addWidget(sectionWidget)
            
        #self.contentLayout.addStretch(1)
        self.scrollArea.setWidget(self.contentWidget)
        
    def sizeHint(self) -> QSize:
        # Suggest a larger size
        return QSize(800, 800)  # Adjust these values as needed


    def updateLabels(self):
        for sectionWidget in self.sectionWidgets:
            for childWidget in sectionWidget.childWidgets:
                childWidget.updateLabels()
        
    