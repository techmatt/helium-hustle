
from __future__ import annotations
from re import M

import sys
import os

from itertools import chain

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import GameDatabase
from game.core.gameState import GameState
from game.core.gameProgram import GameProgram, GameCommand

from game.util.enums import GameWindowMode
from game.util.styleSheets import StyleSheets
from game.ui.mainMenuWidget import MainMenuWidget
from game.ui.resourceDisplayWidget import ResourceDisplayWidget
from game.ui.programWidget import ProgramWidget
from game.ui.eventDialog import EventDialog
from game.ui.eventListWidget import EventListWidget
from game.ui.gameSpeedWidget import GameSpeedWidget

from game.views.commandView import CommandView
from game.views.buildingView import BuildingView
from game.views.researchView import ResearchView
from game.views.projectView import ProjectView
from game.views.ideologyView import IdeologyView
from game.views.statsView import StatsView

from game.util.pixmapCache import PixmapCache
from game.util.formatting import formatSystemUptime

from game.ui.UIWidgets import ProgramUIElements

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
        self.gameSpeed = 1

        self.pixmapCache = PixmapCache()
        
        self.initUI()

    def moveEvent(self, event):
        super().moveEvent(event)
        self.startResizeTimer()

    def resizeEvent(self, event):
        super().resizeEvent(event)
        self.startResizeTimer()
        
    def startResizeTimer(self):
        self.resizeTimer.start(200)  # Wait for 200 ms of inactivity
        
    def initUI(self):
        
        self.activeDialog = None
        self.centralWidget = QWidget()
        self.setCentralWidget(self.centralWidget)
        self.mainLayout = QHBoxLayout(self.centralWidget)
        
        # UI has left, middle, and right frames.
        # See design.pptx for the latest design.

        self.programUIElements = ProgramUIElements(self)

        self.leftFrame = QWidget()
        self.leftLayout = QVBoxLayout(self.leftFrame)
        
        self.middleFrame = QWidget()
        self.middleFrame.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        self.middleLayout = QVBoxLayout(self.middleFrame)
        
        self.rightFrame = QWidget()
        self.rightFrame.setFixedWidth(500)
        self.rightLayout = QVBoxLayout(self.rightFrame)
        
        self.makeLeftFrame()
        self.makeMiddleFrame()
        self.makeRightFrame()
        
        self.mainLayout.addWidget(self.leftFrame, 1)
        self.mainLayout.addWidget(self.middleFrame, 3)
        self.mainLayout.addWidget(self.rightFrame, 1)
        
        self.tickTimer = QTimer(self)
        self.tickTimer.timeout.connect(self.timerTick)
        self.tickTimer.start(self.params.timerInterval)
        
        self.resizeTimer = QTimer(self)
        self.resizeTimer.setSingleShot(True)
        self.resizeTimer.timeout.connect(self.majorUIUpdate)

        
    def clearLayout(self, layout):
        while layout.count():
            item = layout.takeAt(0)
            widget = item.widget()
            if widget:
                widget.setParent(None)

    def majorUIUpdate(self):
        # called when a large update is needed, such as a menu change or an event
        self.makeMiddleFrame()
        self.state.dirty.projects = False
        
    def makeLeftFrame(self):
        self.clearLayout(self.leftLayout)
        
        self.titleLabel = QLabel("Helium Hustle")
        self.titleLabel.setStyleSheet(StyleSheets.GAME_TITLE)
        self.mainMenu = MainMenuWidget(self)
        self.gameSpeedWidget = GameSpeedWidget(self)
        self.resourceDisplay = ResourceDisplayWidget(self)
        
        self.gameSpeedLabel = QLabel("Game Speed")
        self.gameSpeedLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        self.gameTimeWidget = QLabel("Uptime: ")
        self.gameTimeWidget.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)
        
        self.leftLayout.addWidget(self.titleLabel)
        self.leftLayout.addWidget(self.mainMenu)
        self.leftLayout.addWidget(self.gameSpeedLabel)
        self.leftLayout.addWidget(self.gameSpeedWidget)
        self.leftLayout.addWidget(self.gameTimeWidget)
        self.leftLayout.addWidget(self.resourceDisplay)
        
        self.leftLayout.addStretch()

    def makeRightFrame(self):
        self.clearLayout(self.rightLayout)
        
        self.rightLayout.addWidget(self.programUIElements.programLabel)
        self.rightLayout.addWidget(self.programUIElements.programSelectWidget)
        self.rightLayout.addWidget(self.programUIElements.processorAllocationWidget)
        
        self.programWidget = ProgramWidget(self)
        self.rightLayout.addWidget(self.programWidget)
        
        self.eventLabel = QLabel("Events")
        self.eventLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        self.rightLayout.addWidget(self.eventLabel)
        
        self.eventList = EventListWidget(self)
        self.rightLayout.addWidget(self.eventList)
        
        self.rightLayout.addStretch()

    def makeMiddleFrame(self):
        self.clearLayout(self.middleLayout)
        self.commandView = None
        self.buildingView = None
        self.researchView = None
        self.projectView = None
        self.statsView = None

        # TODO: delete these
        middleTitle = "None"
        middleWidget = None
        
        if self.mode == GameWindowMode.COMMANDS:
            middleTitle = "Commands"
            self.commandView = CommandView(self)
            middleWidget = self.commandView.mainWidget
            
        if self.mode == GameWindowMode.BUILDINGS:
            middleTitle = "Buildings"
            self.buildingView = BuildingView(self)
            middleWidget = self.buildingView.mainWidget
            
        if self.mode == GameWindowMode.RESEARCH:
            middleTitle = "Research"
            self.researchView = ResearchView(self)
            middleWidget = self.researchView.mainWidget

        if self.mode == GameWindowMode.PROJECTS:
            middleTitle = "Projects"
            self.projectView = ProjectView(self)
            middleWidget = self.projectView.mainWidget
            
        if self.mode == GameWindowMode.IDEOLOGIES:
            middleTitle = "Ideologies"
            self.ideologyView = IdeologyView(self)
            middleWidget = self.ideologyView.mainWidget

        if self.mode == GameWindowMode.STATS:
            middleTitle = "Statistics and Buffs"
            self.statsView = StatsView(self)
            middleWidget = self.statsView.mainWidget
            
            
        
        self.middleTitleLabel = QLabel(middleTitle)
        self.middleTitleLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        self.middleLayout.addWidget(self.middleTitleLabel)
        middleWidget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        self.middleLayout.addWidget(middleWidget)
        #self.middleLayout.addStretch(1)

    def timerTick(self):
        
        # do not tick game while dialog is active.
        if self.activeDialog:
            return
            
        if self.gameSpeed > 0:
            for i in range(0, self.gameSpeed):
                self.state.step()

        majorUpdateNeeded = False
        if self.state.dirty.projects:
            majorUpdateNeeded = True
            
        if majorUpdateNeeded:
            self.majorUIUpdate()
        else:
            self.updateLabels()
        
        for eState in chain(self.state.activeEvents, self.state.ongoingEvents):
            if not eState.displayed:
                self.displayEvent(eState)
                break
        
    def updateGameTime(self):
        gameSeconds = self.state.ticks * self.database.params.gameSecondsPerTick
        self.gameTimeWidget.setText('System uptime: ' + formatSystemUptime(gameSeconds))
    
    def updateLabels(self):
        self.resourceDisplay.updateLabels()
        self.updateGameTime()
        
        if self.mode == GameWindowMode.COMMANDS:
            self.commandView.updateLabels()
        if self.mode == GameWindowMode.BUILDINGS:
            self.buildingView.updateLabels()
        if self.mode == GameWindowMode.RESEARCH:
            self.researchView.updateLabels()
        if self.mode == GameWindowMode.PROJECTS:
            self.projectView.updateLabels()
        if self.mode == GameWindowMode.IDEOLOGIES:
            self.ideologyView.updateLabels()
        if self.mode == GameWindowMode.STATS:
            self.statsView.updateLabels()
            
        self.programWidget.updateProgram()
        self.programWidget.updateProgressBars()
        self.programUIElements.updateVisisbleProgramIndex()
        
        if self.state.dirty.events:
            self.updateEventList()
            
    def updateEventList(self):
        # Find and remove the old EventListWidget
        for i in range(self.rightLayout.count()):
            widget = self.rightLayout.itemAt(i).widget()
            if isinstance(widget, EventListWidget):
                self.rightLayout.removeWidget(widget)
                widget.deleteLater()
                break
    
        # Create and add the new EventListWidget
        self.eventList = EventListWidget(self)
        self.rightLayout.insertWidget(self.rightLayout.count() - 1, self.eventList)
        self.state.dirty.events = False

    def runCommand(self, name : str):
        self.state.runCommand(name)
        self.updateLabels()
        
    def purchaseBuilding(self, name : str):
        self.state.attemptPurchaseBuilding(name)
        self.updateLabels()
        
    def purchaseResearch(self, name : str):
        self.state.attemptPurchaseResearch(name)
        self.updateLabels()
        
    def makeIconLabel(self, iconPath, iconWidth, iconHeight):
        result = QLabel()
        result.setPixmap(self.pixmapCache.getPixmap(iconPath, iconWidth, iconHeight))
        result.setFixedSize(iconWidth, iconHeight)
        return result
    
    def makeIconButton(self, iconPath, iconWidth, iconHeight):
        result = QPushButton()
        icon = QIcon(self.pixmapCache.getPixmap(iconPath, iconWidth, iconHeight))
        result.setIcon(icon)
        result.setIconSize(QSize(iconWidth, iconHeight))
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

    def modifyProjectPayment(self, pName, rName, value):
        pState : ProjectState = self.state.projects[pName]
        pState.resourcePayments[rName] = max(0, pState.resourcePayments[rName] + value)
        
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
        
    def displayEvent(self, eState : EventState):
        #print('display event: ' + eState.info.name)
        eState.displayed = True
        self.activeDialog = EventDialog(self, self, eState)
        self.activeDialog.show()
        
    def triggerExit(self):
        QCoreApplication.instance().quit()
