from __future__ import annotations
from PyQt6.QtWidgets import QDialog, QHBoxLayout, QVBoxLayout, QLabel, QPushButton, QFrame, QSizePolicy, QLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, QSize

from functools import partial

from game.util.styleSheets import StyleSheets

class EventDialog(QDialog):
    def __init__(self, parent, gameUI : GameUI, eState : EventState):
        super().__init__(parent)
        
        self.gameUI = gameUI
        self.eState = eState
        eInfo = eState.info
        
        self.setWindowFlag(Qt.WindowType.Window | Qt.WindowType.FramelessWindowHint)
        #self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setWindowModality(Qt.WindowModality.ApplicationModal)
        self.setSizePolicy(QSizePolicy.Policy.Preferred, QSizePolicy.Policy.Minimum)
        self.minWidth = 600
        self.setMinimumWidth(self.minWidth)
        
        mainLayout = QVBoxLayout(self)
        mainLayout.setSizeConstraint(QLayout.SizeConstraint.SetMinimumSize)
        
        titleBar = QFrame()
        #titleBar.setFixedHeight(60)
        titleBar.setStyleSheet("""
            background-color: #3498db;
            border-top-left-radius: 10px;
            border-top-right-radius: 10px;
        """)
        titleBarLayout = QHBoxLayout(titleBar)
        
        # Title label with custom font
        titleLabel = QLabel(eInfo.name)
        titleFont = QFont("Segoe UI", 14, QFont.Weight.Bold)
        titleLabel.setFont(titleFont)
        titleLabel.setStyleSheet("color: white;")
        titleBarLayout.addWidget(titleLabel)
        mainLayout.addWidget(titleBar)
        
        contentFrame = QFrame()
        contentFrame.setStyleSheet("""
            background-color: white;
            border-bottom-left-radius: 10px;
            border-bottom-right-radius: 10px;
            border: 1px solid #e0e0e0;
        """)
        contentLayout = QVBoxLayout(contentFrame)
        
        # Flavor Text
        flavorLabel = QLabel(eInfo.flavorText)
        flavorLabel.setWordWrap(True)
        flavorLabel.setStyleSheet("""
            color: #333;
            font-size: 16px;
            margin-bottom: 20px;
        """)
        contentLayout.addWidget(flavorLabel)
        
        # Mechanics Text (now bold and italic)
        mechanicsLabel = QLabel(f"<b><i>{eInfo.mechanicsText}</i></b>")
        mechanicsLabel.setWordWrap(True)
        mechanicsLabel.setStyleSheet("""
            color: #333;
            font-size: 16px;
            margin-bottom: 20px;
        """)
        contentLayout.addWidget(mechanicsLabel)
        
        # Separator
        #separator = QFrame()
        #separator.setFrameShape(QFrame.Shape.HLine)
        #separator.setStyleSheet("background-color: #e0e0e0;")
        #contentLayout.addWidget(separator)
        
        buttonLayout = QHBoxLayout()

        options = eInfo.options
        if len(options) == 0 or eState.completed:
            options = ["OK"]
            
        buttonLayout.addStretch()
        for o in options:
            button = QPushButton(o)
            button.clicked.connect(partial(self.buttonPressed, o))
            button.setStyleSheet(StyleSheets.DIALOG_BUTTON)
            buttonLayout.addWidget(button)
        buttonLayout.addStretch()
            
        contentLayout.addLayout(buttonLayout)
        contentLayout.addStretch()
        
        mainLayout.addWidget(contentFrame)
        
        # Set a custom font for the entire dialog
        dialogFont = QFont("Segoe UI", 10)
        self.setFont(dialogFont)
        
        # Set frame style for border
        self.setStyleSheet("""
            QDialog {
                background-color: transparent;
                border: 2px solid #e0e0e0;
                border-radius: 10px;
            }
        """)
        
        self.adjustSize()
        
    def sizeHint(self) -> QSize:
        # Suggest a size that respects the minimum width but allows for minimum height
        return QSize(max(self.minWidth, super().sizeHint().width()), 0)
        
    def buttonPressed(self, text):
        self.gameUI.state.processEventOption(self.eState, text)
        
        #print('dialog', self.gameUI.dialog)
        self.gameUI.activeDialog = None
        self.accept()
            

        


