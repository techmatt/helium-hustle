
from __future__ import annotations

import random
from functools import partial

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton,
    QLabel, QGridLayout, QSizePolicy, QScrollArea, QFrame, QSpacerItem )
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
    def __init__(self, gameUI, sectionEntry, layoutType):
        super().__init__()
        self.gameUI = gameUI
        self.childWidgets = sectionEntry.childWidgets
        self.title = sectionEntry.title
        self.layoutType = layoutType
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(2)

        self.toggleButton = QPushButton(self.title)
        self.toggleButton.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)
        self.toggleButton.setCheckable(True)
        self.toggleButton.setChecked(True)
        self.toggleButton.clicked.connect(self.toggleContent)
        self.updateArrow()
        layout.addWidget(self.toggleButton)

        self.contentWidget = QWidget()
        self.contentWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        
        if self.layoutType == 'grid':
            self.contentLayout = QGridLayout(self.contentWidget)
            self.contentLayout.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)
            self.contentWidget.resizeEvent = self.onResize
        elif self.layoutType == 'list':
            self.contentLayout = QVBoxLayout(self.contentWidget)
            for childWidget in self.childWidgets:
                self.contentLayout.addWidget(childWidget)
        else:
            print(f"Invalid layout type: {self.layoutType}")
    
        layout.addWidget(self.contentWidget)

        if self.layoutType == 'grid':
            self.updateGridLayout()

    def toggleContent(self):
        self.contentWidget.setVisible(self.toggleButton.isChecked())
        self.updateArrow()

    def updateArrow(self):
        arrowText = ' \u25C0' if not self.toggleButton.isChecked() else ' \u25BC'
        self.toggleButton.setText(self.title + arrowText)

    def onResize(self, event):
        super().resizeEvent(event)
        self.updateGridLayout()

    def updateGridLayout(self):
        if self.layoutType != 'grid':
            return

        width = self.contentWidget.width()
        columnCount = max(1, width // 275)

        # Remove all widgets and spacer items from the layout
        for i in reversed(range(self.contentLayout.count())): 
            item = self.contentLayout.itemAt(i)
            if item.widget():
                self.contentLayout.removeWidget(item.widget())
            elif isinstance(item, QSpacerItem):
                self.contentLayout.removeItem(item)

        # Add child widgets to the grid
        for index, childWidget in enumerate(self.childWidgets):
            row = index // columnCount
            col = index % columnCount
            self.contentLayout.addWidget(childWidget, row, col)

        # Add placeholder widgets if needed
        """totalItems = len(self.childWidgets)
        if totalItems % columnCount != 0:
            placeholdersNeeded = columnCount - (totalItems % columnCount)
            for i in range(placeholdersNeeded):
                placeholder = QWidget()
                placeholder.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
                self.contentLayout.addWidget(placeholder, totalItems // columnCount, (totalItems % columnCount) + i)"""

        # Update the layout
        #self.contentLayout.update()

class CollapsibleMenuWidget(QWidget):
    def __init__(self, gameUI : GameUI, sectionEntries : Dict[str, CollapsibleSectionEntries], layoutType : str):
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
        mainLayout.addWidget(self.scrollArea)

        # Create collapsible sections
        
        self.contentWidget = QWidget()
        self.contentLayout = QVBoxLayout(self.contentWidget)
        self.contentWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        
        self.sectionWidgets : List[CollapsibleSectionWidget] = []
        for sectionEntry in sectionEntries.values():
            sectionWidget = CollapsibleSectionWidget(gameUI, sectionEntry, layoutType)
            sectionWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)

            self.sectionWidgets.append(sectionWidget)
            self.contentLayout.addWidget(sectionWidget)
            
        self.contentLayout.addStretch(1)
        self.scrollArea.setWidget(self.contentWidget)
        
    def updateLabels(self):
        for sectionWidget in self.sectionWidgets:
            for childWidget in sectionWidget.childWidgets:
                childWidget.updateLabels()
        
    