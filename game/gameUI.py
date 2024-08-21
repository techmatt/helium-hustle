
from __future__ import annotations

import sys
import os

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from gameDatabase import GameDatabase
from gameState import GameState
from gameProgram import GameProgram, GameCommand

from enums import GameWindowMode
from iconGrid import IconGrid
from resourceDisplay import ResourceDisplay
from styleSheets import StyleSheets
from programWidget import ProgramWidget
from pixmapCache import PixmapCache
from commandViewWidget import CommandViewWidget
from buildingViewWidget import BuildingViewWidget

from UIWidgets import ProgramUIElements

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

        self.programUIElements = ProgramUIElements(self)

        self.leftFrame = QWidget()
        self.leftLayout = QVBoxLayout(self.leftFrame)
        
        self.middleFrame = QWidget()
        self.middleLayout = QVBoxLayout(self.middleFrame)
        
        self.rightFrame = QWidget()
        self.rightFrame.setFixedWidth(532)
        self.rightLayout = QVBoxLayout(self.rightFrame)
        
        self.makeLeftFrame()
        self.makeMiddleFrame()
        self.makeRightFrame()
        
        self.mainLayout.addWidget(self.leftFrame, 1)
        self.mainLayout.addWidget(self.middleFrame, 2)
        self.mainLayout.addStretch(1)
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
        
        self.rightLayout.addWidget(self.programUIElements.programLabel)
        self.rightLayout.addWidget(self.programUIElements.programSelectWidget)
        self.rightLayout.addWidget(self.programUIElements.processorAllocationWidget)
        
        self.programWidget = ProgramWidget(self)
        self.rightLayout.addWidget(self.programWidget)
        
        self.rightLayout.addStretch()

    def makeMiddleFrame(self):
        self.clearLayout(self.middleLayout)
        self.commandViewWidget = None
        self.buildingViewWidget = None
        
        if self.mode == GameWindowMode.COMMANDS:
            self.commandViewWidget = CommandViewWidget(self)
            self.middleLayout.addWidget(self.commandViewWidget)

        if self.mode == GameWindowMode.BUILDINGS:
            self.buildingViewWidget = BuildingViewWidget(self)
            self.middleLayout.addWidget(self.buildingViewWidget)
            
        self.middleLayout.addStretch(1)

    def timerTick(self):
        self.state.step()
        self.updateLabels()
        
    def updateLabels(self):
        self.resourceDisplay.updateLabels()
        
        if self.mode == GameWindowMode.COMMANDS:
            self.commandViewWidget.updateLabels()
        if self.mode == GameWindowMode.BUILDINGS:
            self.buildingViewWidget.updateLabels()
        
        #self.makeMiddleFrame()
            
        self.programWidget.updateProgram()
        self.programWidget.updateProgressBars()
        self.programUIElements.updateVisisbleProgramIndex()
    
    def runCommand(self, name : str):
        self.state.runCommand(name)
        self.updateLabels()
        
    def buildBuilding(self, name : str):
        self.state.attemptPurchaseBuilding(name)
        self.updateLabels()
        
    def makeIconLabel(self, iconPath, iconWidth, iconHeight):
        result = QLabel()
        result.setPixmap(self.pixmapCache.getPixmap(iconPath, iconWidth, iconHeight))
        result.setFixedSize(iconWidth, iconHeight)
        return result

    def changeVisibleProgramIndex(self, i):
        self.visibleProgramIndex = i
        self.updateLabels()

    def changeAssignedProcessors(self, delta):
        activeProgram = self.getActiveProgram()
        if delta < 0:
            activeProgram.assignedProcessors = max(0, activeProgram.assignedProcessors + delta)
        else:
            maxAvailableProcessors = min(delta, self.state.freeProcessorCount)
            activeProgram.assignedProcessors += maxAvailableProcessors
        self.updateLabels()

    def restartAllPrograms(self):
        for program in self.state.programs:
            program.resetAllCommands()
        self.updateLabels()

    def getActiveProgram(self) -> GameProgram:
        return self.state.programs[self.visibleProgramIndex]
    
    def addCommandToProgram(self, commandName):
        activeProgram = self.getActiveProgram()
        activeProgram.commands.append(GameCommand(self.state.commands[commandName].info))
        self.updateLabels()

    def modifyBuildingActive(self, bName : str, deltaValue : int):
        bState : BuildingState = self.state.buildings[bName]
        bState.activeCount = bState.activeCount + deltaValue
        bState.activeCount = max(bState.activeCount, 0)
        bState.activeCount = min(bState.activeCount, bState.totalCount)
        self.updateLabels()
        
    def removeBuilding(self, bName : str):
        bState : BuildingState = self.state.buildings[bName]
        bState.totalCount -= 1
        bState.totalCount = max(bState.totalCount, 0)
        bState.activeCount = min(bState.activeCount, bState.totalCount)
        self.updateLabels()
        
    def triggerExit(self):
        QCoreApplication.instance().quit()

if __name__ == '__main__':
    print('starting UI')
    
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameData')
    state = GameState(database)
    
    app = QApplication(sys.argv)
    game = GameUI(state, database)
    game.show()
    sys.exit(app.exec())