from __future__ import annotations

from functools import partial

from PyQt6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                             QScrollArea, QFrame, QPushButton)
from PyQt6.QtCore import Qt, QSize
from PyQt6.QtGui import QFont

from game.util.styleSheets import StyleSheets

class EventListWidget(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        self.initUI()

    def initUI(self):
        mainLayout = QVBoxLayout()
        self.setLayout(mainLayout)

        mainLayout.setContentsMargins(2, 2, 2, 2)
        mainLayout.setSpacing(2)

        # Create a frame around the whole widget
        frame = QFrame()
        frame.setFrameShape(QFrame.Shape.StyledPanel)
        frameLayout = QVBoxLayout(frame)

        # Create the three collapsible sections
        self.activeSection = EventCollapsibleSection("Active Events")
        self.ongoingSection = EventCollapsibleSection("Ongoing Events")
        self.finishedSection = EventCollapsibleSection("Finished Events")

        frameLayout.addWidget(self.activeSection)
        frameLayout.addWidget(self.ongoingSection)
        frameLayout.addWidget(self.finishedSection)

        mainLayout.addWidget(frame)

        self.loadAllEvents()

    def loadAllEvents(self):
        state : GameState = self.gameUI.state
        for e in state.events.values():
            # events have the following booleans: completed, displayed, triggered, ongoing
            if not e.triggered:
                continue
            if e.ongoing:
                self.addEvent(self.ongoingSection, e)
            elif e.completed:
                self.addEvent(self.finishedSection, e)
            else:
                self.addEvent(self.activeSection, e)
                

    def addEvent(self, section, eState : EventState):
        eventWidget = EventWidget(eState)
        eventWidget.clicked.connect(partial(self.gameUI.displayEvent, eState))
        section.addWidget(eventWidget)

class EventCollapsibleSection(QWidget):
    def __init__(self, title):
        super().__init__()
        self.initUI(title)

    def initUI(self, title):
        layout = QVBoxLayout()
        self.setLayout(layout)

        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(2)

        # Create a button for expanding/collapsing with an arrow icon
        self.title = title
        self.toggleButton = QPushButton(title)
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
        self.contentLayout = QVBoxLayout(self.contentWidget)
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

class EventWidget(QPushButton):
    def __init__(self, eState):
        text = f"{eState.timestampStr} - {eState.info.name}"
        super().__init__()
        self.setText(text)
        self.setStyleSheet("""
            QPushButton {
                background-color: transparent;
                border: none;
                text-align: left;
                padding: 5px;
                font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
                font-size: 12pt;
                color: #2c3e50;
                font-weight: 400;
                line-height: 1.4;
            }
            QPushButton:hover {
                background-color: rgba(200, 200, 200, 50);
            }
        """)
