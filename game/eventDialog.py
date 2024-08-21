
from __future__ import annotations

from PyQt6.QtWidgets import QDialog, QVBoxLayout, QLabel, QPushButton, QFrame
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt

class EventDialog(QDialog):
    def __init__(self, parent, gameUI : GameUI, eState : EventState):
        super().__init__(parent)
        
        self.gameUI = gameUI
        self.eState = eState

        eInfo = eState.info
        
        self.setWindowTitle(eInfo.name)
        self.setWindowModality(Qt.WindowModality.ApplicationModal)
        
        layout = QVBoxLayout()
        
        # Event text
        flavorLabel = QLabel(eInfo.flavorText)
        flavorLabel.setWordWrap(True)
        layout.addWidget(flavorLabel)
        
        flavorLabel.setStyleSheet("""
            color: #333;
            font-size: 16px;
            margin-bottom: 20px;
        """)

        separator = QFrame()
        separator.setFrameShape(QFrame.Shape.HLine)
        separator.setStyleSheet("background-color: #e0e0e0;")
        layout.addWidget(separator)
        
        mechanicsLabel = QLabel(eInfo.mechanicsText)
        mechanicsLabel.setWordWrap(True)
        layout.addWidget(mechanicsLabel)
        
        # OK button
        OKButton = QPushButton("OK")
        OKButton.clicked.connect(self.accept)
        layout.addWidget(OKButton)
        
        OKButton.setStyleSheet("""
            QPushButton {
                background-color: #e74c3c;
                color: white;
                border: none;
                padding: 10px;
                font-size: 16px;
                font-weight: bold;
                border-radius: 5px;
            }
            QPushButton:hover {
                background-color: #c0392b;
            }
        """)
        
        self.setLayout(layout)
        
        self.setStyleSheet("""
            QDialog {
                background-color: white;
                border-radius: 10px;
            }
        """)
        
        # Set a custom font for the entire dialog
        font = QFont("Segoe UI", 14)
        self.setFont(font)
        
        # Set a fixed size for the dialog
        self.setFixedSize(400, 300)

