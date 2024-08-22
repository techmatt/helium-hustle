from __future__ import annotations

from PyQt6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                             QScrollArea, QFrame, QPushButton)
from PyQt6.QtCore import Qt, QSize
from PyQt6.QtGui import QFont

from styleSheets import StyleSheets

class EventList(QWidget):
    def __init__(self):
        super().__init__()
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
        self.activeSection = CollapsibleSection("Active Events")
        self.ongoingSection = CollapsibleSection("Ongoing Events")
        self.finishedSection = CollapsibleSection("Finished Events")

        frameLayout.addWidget(self.activeSection)
        frameLayout.addWidget(self.ongoingSection)
        frameLayout.addWidget(self.finishedSection)

        mainLayout.addWidget(frame)

        # Add sample events
        for i in range(20):  # Add 20 events to each section for scrolling demo
            self.add_event(self.activeSection, f"Active Event {i}", f"2023-05-{i:02d}", f"Description of active event {i}")
            self.add_event(self.ongoingSection, f"Ongoing Event {i}", f"2023-05-{i:02d}", f"Description of ongoing event {i}")
            self.add_event(self.finishedSection, f"Finished Event {i}", f"2023-05-{i:02d}", f"Description of finished event {i}")

    def add_event(self, section, name, timestamp, description):
        event_widget = EventWidget(name)
        section.addWidget(event_widget)

class CollapsibleSection(QWidget):
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
    def __init__(self, text):
        super().__init__(text)
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
