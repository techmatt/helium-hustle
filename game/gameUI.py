
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
from styleSheets import StyleSheets

from UIWidgets import CommandWidget, BuildingWidget

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
                cWidget = CommandWidget(self, cName)
                commandListLayout.addWidget(cWidget)
                
            self.middleLayout.addWidget(commandListWidget)
                
        if self.mode == GameWindowMode.BUILDINGS:
            buildingGridWidget = QWidget()
            buildingGridLayout = QGridLayout(buildingGridWidget)
        
            for bName in self.database.buildings.keys():
                bWidget = BuildingWidget(self.state, bName)
                buildingGridLayout.addWidget(bWidget)
                bWidget.clicked.connect(self.buildBuilding)
            
            self.middleLayout.addWidget(buildingGridWidget)
            
        self.middleLayout.addStretch()

    def timerTick(self):
        self.state.step()
        #self.updateLabels()
        
    #self.cookiesLabel.setText(f"Cookies: {self.cookies}\nPer click: {self.cookiesPerClick}\nPer second: {self.cookiesPerSecond}")
    #upgradeClickBtn = QPushButton("Upgrade Click (+1)")
    
    def runCommand(self, name : str):
        print('running ' + name)
        
    def buildBuilding(self, name : str):
        print('building ' + name)
        
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