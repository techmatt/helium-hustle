
from __future__ import annotations

import sys
import os

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import GameDatabase
from gameState import GameState

from enums import GameWindowMode
from iconGrid import IconGrid
from resourceDisplay import ResourceDisplay

class CommandWidget(QWidget):
    clicked = pyqtSignal(str)
    
    def __init__(self, state : GameState, name : str):
        super().__init__()
        
        self.name = name
    
        layout = QHBoxLayout()
        
        cState = state.commands[name]
        
        # Name
        nameLabel = QLabel(f"{name}")
        nameLabel.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        layout.addWidget(nameLabel)
            
        self.setLayout(layout)
        
    def mousePressEvent(self, event):
        self.clicked.emit(self.name)
        
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

class GameUI(QMainWindow):
    def __init__(self, state : GameState, database : GameDatabase):
        super().__init__()
        self.setWindowTitle("Helium Hustle")
        self.setGeometry(100, 100, 600, 400)

        self.state = state
        self.database = database
        self.params = self.database.params
        self.mode : GameWindowMode = GameWindowMode.BUILDINGS
        
        self.initUI()

    def initUI(self):
        self.centralWidget = QWidget()
        self.setCentralWidget(self.centralWidget)
        self.mainLayout = QHBoxLayout(self.centralWidget)
        
        # UI has left, middle, and right frames.
        # See design.pptx for the latest design.

        self.leftFrame = QWidget()
        self.leftLayout = QVBoxLayout(self.leftFrame)
        
        self.middleFrame = QWidget()
        self.middleLayout = QVBoxLayout(self.middleFrame)
        
        self.rightFrame = QWidget()
        self.rightLayout = QVBoxLayout(self.rightFrame)
        
        self.makeLeftFrame()
        self.makeMiddleFrame()
        self.makeRightFrame()
        
        self.mainLayout.addWidget(self.leftFrame, 1)
        self.mainLayout.addWidget(self.middleFrame, 2)
        self.mainLayout.addWidget(self.rightFrame, 3)
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.timerTick)
        self.timer.start(self.params.timerInterval)
        
    def clearLayout(self, layout):
        while layout.count():
            item = layout.takeAt(0)
            widget = item.widget()
            if widget:
                widget.setParent(None)

    def makeLeftFrame(self):
        self.clearLayout(self.leftLayout)
        
        self.titleLabel = QLabel("Helium Hustle")
        self.iconGrid = IconGrid(self)
        self.resourceDisplay = ResourceDisplay(self)
        
        self.leftLayout.addWidget(self.titleLabel)
        self.leftLayout.addWidget(self.iconGrid)
        self.leftLayout.addWidget(self.resourceDisplay)
        
        self.leftLayout.addStretch()

    def makeRightFrame(self):
        self.clearLayout(self.rightLayout)
        
        self.programLabel = QLabel("Program")

        self.rightLayout.addWidget(self.programLabel)
        
        self.rightLayout.addStretch()

    def makeMiddleFrame(self):
        self.clearLayout(self.middleLayout)
        if self.mode == GameWindowMode.COMMANDS:
            commandListWidget = QWidget()
            commandListLayout = QVBoxLayout(commandListWidget)
            
            for cName in self.database.commands.keys():
                cWidget = CommandWidget(self.state, cName)
                commandListLayout.addWidget(cWidget)
                cWidget.clicked.connect(self.runCommand)

            self.middleLayout.addWidget(commandListWidget)
                
        if self.mode == GameWindowMode.BUILDINGS:
            buildingGridWidget = QWidget()
            buildingGridLayout = QGridLayout(buildingGridWidget)
        
            for bName in self.database.buildings.keys():
                bWidget = BuildingWidget(self.state, bName)
                buildingGridLayout.addWidget(bWidget)
                bWidget.clicked.connect(self.buildBuilding)
            
            self.middleLayout.addWidget(buildingGridWidget)

    def timerTick(self):
        self.state.step()
        #self.updateLabels()
        
    #def updateCookiesLabel(self):
    #    self.cookiesLabel.setText(f"Cookies: {self.cookies}\nPer click: {self.cookiesPerClick}\nPer second: {self.cookiesPerSecond}")

    def showUpgrades(self):
        # Clear right layout and add upgrade widgets
        self.clearRightLayout()
        
        buildingGridWidget = QWidget()
        buildingGridLayout = QGridLayout(buildingGridWidget)
        
        for bName in self.database.buildings.keys():
            bWidget = BuildingWidget(self.state, bName)
            buildingGridLayout.addWidget(bWidget)
            bWidget.clicked.connect(self.buildBuilding)
            
        self.rightLayout.addWidget(buildingGridWidget)
        
        #upgradeClickBtn = QPushButton("Upgrade Click (+1)")
        #upgradePassiveBtn = QPushButton("Upgrade Passive (+1/s)")
        #self.rightLayout.addWidget(upgradeClickBtn)
        #self.rightLayout.addWidget(upgradePassiveBtn)
        #upgradeClickBtn.clicked.connect(lambda: self.upgrade("click"))
        #upgradePassiveBtn.clicked.connect(lambda: self.upgrade("passive"))

    def runCommand(self, name : str):
        print('running ' + name)
        
    def buildBuilding(self, name : str):
        print('building ' + name)
        
    def upgrade(self, upgradeType):
        if upgradeType == "click" and self.cookies >= 10:
            self.cookies -= 10
            self.cookiesPerClick += 1
        elif upgradeType == "passive" and self.cookies >= 20:
            self.cookies -= 20
            self.cookiesPerSecond += 1
        self.updateCookiesLabel()
        
    def triggerExit(self):
        QCoreApplication.instance().quit()

if __name__ == '__main__':
    print('starting UI')
    
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    
    app = QApplication(sys.argv)
    game = GameUI(state, database)
    game.show()
    sys.exit(app.exec())