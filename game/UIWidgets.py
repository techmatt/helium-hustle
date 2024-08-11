
from __future__ import annotations

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import GameDatabase
from gameState import GameState

from enums import GameWindowMode
from iconGrid import IconGrid
from resourceDisplay import ResourceDisplay
from styleSheets import StyleSheets

class CommandWidget(QWidget):
    #clicked = pyqtSignal(str)
    
    def __init__(self, gameUI : GameUI, name : str):
        super().__init__()
        
        self.name = name
    
        layout = QHBoxLayout()
        
        cState = gameUI.state.commands[name]
        
        # Name
        nameButton = QPushButton(f"{name}")
        nameButton.setStyleSheet(StyleSheets.MODERN_BUTTON)
        #nameButton.clicked.connect(self.button_clicked)
        nameButton.clicked.connect(lambda: gameUI.runCommand(name))

        #nameLabel.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        layout.addWidget(nameButton)

        self.setLayout(layout)
        
    #def mousePressEvent(self, event):
    #    self.clicked.emit(self.name)

class BuildingButton(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, state : GameState, name : str):
        super().__init__()
        self.name = name
        self.setFixedSize(150, 150)  # Adjust size as needed
        
        bState = state.buildings[name]
        buildingCost = state.getBuildingCost(name)

        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)

        # Icon
        iconLabel = QLabel()
        iconPath = 'icons/buildings/' + name + '.png'
        pixmap = QPixmap(iconPath).scaled(
            64, 64, 
            Qt.AspectRatioMode.KeepAspectRatio, 
            Qt.TransformationMode.SmoothTransformation
        )
        iconLabel.setPixmap(pixmap)
        iconLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        
        # Name and count
        textLabel = QLabel(f"{name} ({bState.count})")
        textLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        
        # Cost
        #for rName, cost in buildingCost.costs.items():
        #    costLabel = QLabel(f"{rName} {cost}")
        #    layout.addWidget(costLabel)

        #canAfford = state.canAffordCost(buildingCost)
        #color = "#90EE90"
        #if not canAfford:
        #    color = "#F08080"

        layout.addWidget(textLabel)
        layout.addWidget(iconLabel)

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        if self.underMouse():
            painter.setBrush(QColor(200, 200, 200, 100))
        else:
            painter.setBrush(QColor(230, 230, 230, 100))

        painter.setPen(QColor(180, 180, 180))
        painter.drawRoundedRect(self.rect(), 10, 10)

    def enterEvent(self, event):
        self.update()

    def leaveEvent(self, event):
        self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.name)

"""class BuildingWidget(QWidget):
    clicked = pyqtSignal(str)
    
    def __init__(self, state : GameState, name : str):
        super().__init__()
        
        self.name = name
    
        layout = QVBoxLayout()
        
        bState = state.buildings[name]
        buildingCost = state.getBuildingCost(name)

        # Name
        nameLabel = QLabel(f"{name} ({bState.count})")
        nameLabel.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        layout.addWidget(nameLabel)
        
        # Cost
        for rName, cost in buildingCost.costs.items():
            costLabel = QLabel(f"{rName} {cost}")
            layout.addWidget(costLabel)

        canAfford = state.canAffordCost(buildingCost)
        color = "#90EE90"
        if not canAfford:
            color = "#F08080"
            
        self.setLayout(layout)
        self.setStyleSheet(f"background-color: {color}; border-radius: 10px; padding: 10px;")
        
    def mousePressEvent(self, event):
        self.clicked.emit(self.name)"""
