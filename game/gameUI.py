
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
from programWidget import ProgramWidget
from pixmapCache import PixmapCache

from UIWidgets import CommandButton, BuildingButton

class GameUI(QMainWindow):
    def __init__(self, state : GameState, database : GameDatabase):
        super().__init__()
        self.setWindowTitle("Helium Hustle")
        self.setGeometry(100, 100, 1900, 1100)

        self.state = state
        self.database = database
        self.params = self.database.params
        self.mode : GameWindowMode = GameWindowMode.BUILDINGS
        self.visibleProgramIndex = 0

        self.pixmapCache = PixmapCache()
        
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
        self.titleLabel.setStyleSheet(StyleSheets.GAME_TITLE)
        self.iconGrid = IconGrid(self)
        self.resourceDisplay = ResourceDisplay(self)
        
        self.leftLayout.addWidget(self.titleLabel)
        self.leftLayout.addWidget(self.iconGrid)
        self.leftLayout.addWidget(self.resourceDisplay)
        
        self.leftLayout.addStretch()

    def makeRightFrame(self):
        self.clearLayout(self.rightLayout)
        
        self.programLabel = QLabel("Programs")
        self.programLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        self.rightLayout.addWidget(self.programLabel)
        
        programSelectWidget = QWidget()
        programSelectLayout = QHBoxLayout(programSelectWidget)
        for i in range(0, len(self.state.programs)):
            programIndexButton = QPushButton(str(i+1))
            programIndexButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
            programSelectLayout.addWidget(programIndexButton)
        self.rightLayout.addWidget(programSelectWidget)
            
        self.programWidget = ProgramWidget(self)
        self.rightLayout.addWidget(self.programWidget)
        
        self.rightLayout.addStretch()

    def makeMiddleFrame(self):
        self.clearLayout(self.middleLayout)
        if self.mode == GameWindowMode.COMMANDS:
            commandGridWidget = QWidget()
            commandGridLayout = QGridLayout(commandGridWidget)
        
            for index, bName in enumerate(self.database.commands.keys()):
                cWidget = CommandButton(self, bName)

                row = index // 3
                col = index % 3
                commandGridLayout.addWidget(cWidget, row, col)
                
                cWidget.clicked.connect(self.runCommand)
            
            self.middleLayout.addWidget(commandGridWidget)

        if self.mode == GameWindowMode.BUILDINGS:
            buildingGridWidget = QWidget()
            buildingGridLayout = QGridLayout(buildingGridWidget)
        
            for index, bName in enumerate(self.database.buildings.keys()):
                bWidget = BuildingButton(self, bName)

                row = index // 3
                col = index % 3
                buildingGridLayout.addWidget(bWidget, row, col)
                
                bWidget.clicked.connect(self.buildBuilding)
            
            self.middleLayout.addWidget(buildingGridWidget)
            
        self.middleLayout.addStretch(1)

    def timerTick(self):
        self.state.step()
        self.updateLabels()
        
    def updateLabels(self):
        self.resourceDisplay.updateLabels()
        self.makeMiddleFrame()
        self.programWidget.updateProgressBars()
        #self.makeRightFrame()
    
    def runCommand(self, name : str):
        #print('running ' + name)
        self.state.runCommand(name)
        self.updateLabels()
        
    def buildBuilding(self, name : str):
        #print('building ' + name)
        self.state.attemptPurchaseBuilding(name)
        self.updateLabels()
        
    def makeIconLabel(self, iconPath, iconWidth, iconHeight):
        result = QLabel()
        result.setPixmap(self.pixmapCache.getPixmap(iconPath, iconWidth, iconHeight))
        result.setFixedSize(iconWidth, iconHeight)
        return result
        
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