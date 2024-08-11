
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
        
class BuildingWidget(QWidget):
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
        self.clicked.emit(self.name)
