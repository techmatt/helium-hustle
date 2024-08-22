
from __future__ import annotations

import random
from functools import partial

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import GameDatabase
from game.core.gameState import GameState

from game.util.styleSheets import StyleSheets

class ProgramUIElements():
    def __init__(self, gameUI : GameUI):
        self.gameUI = gameUI
        
        self.programLabel = QLabel("Programs")
        self.programLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        self.programSelectWidget = QWidget()
        programSelectLayout = QHBoxLayout(self.programSelectWidget)
        self.programIndexButtons = []
        for i in range(0, len(gameUI.state.programs)):
            button = QPushButton(str(i+1))
            button.clicked.connect(partial(gameUI.changeVisibleProgramIndex, i))
            programSelectLayout.addWidget(button)
            self.programIndexButtons.append(button)

        self.processorAllocationWidget = QWidget()
        processorAllocationLayout = QHBoxLayout(self.processorAllocationWidget)
        processorIcon = gameUI.makeIconLabel('icons/resources/processors.png', 32, 32)
        
        activeProgram = gameUI.state.programs[gameUI.visibleProgramIndex]
        self.assignedProcessorsLabel = QLabel("")
        
        processorSubButton = QPushButton("-")
        processorAddButton = QPushButton("+")
        processorRestartButton = QPushButton("Restart all")
        
        processorSubButton.clicked.connect(partial(gameUI.changeAssignedProcessors, -1))
        processorAddButton.clicked.connect(partial(gameUI.changeAssignedProcessors, 1))
        processorRestartButton.clicked.connect(partial(gameUI.restartAllPrograms))

        buttonSize = QSize(25, 25)
        processorSubButton.setFixedSize(buttonSize)
        processorAddButton.setFixedSize(buttonSize)
        
        self.assignedProcessorsLabel.setStyleSheet(StyleSheets.RESOURCE_LIST_TEXT)
        processorSubButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        processorAddButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        processorRestartButton.setStyleSheet(StyleSheets.RESOURCE_LIST_TEXT)

        processorAllocationLayout.addWidget(processorIcon)
        processorAllocationLayout.addWidget(self.assignedProcessorsLabel)
        processorAllocationLayout.addStretch(1)
        processorAllocationLayout.addWidget(processorSubButton)
        processorAllocationLayout.addWidget(processorAddButton)
        processorAllocationLayout.addWidget(processorRestartButton)
        
        self.updateVisisbleProgramIndex()

    def updateVisisbleProgramIndex(self):
        state : GameState = self.gameUI.state
        for i in range(0, len(state.programs)):
            if i == self.gameUI.visibleProgramIndex:
                self.programIndexButtons[i].setStyleSheet(StyleSheets.SELECTED_BUTTON)
            else:
                self.programIndexButtons[i].setStyleSheet(StyleSheets.BUILDING_TITLE)

        activeProgram = state.programs[self.gameUI.visibleProgramIndex]
        self.assignedProcessorsLabel.setText(f"{activeProgram.assignedProcessors} processors assigned ({state.freeProcessorCount} free)")

#rWidget = QLabel("text!")
#color = QColor(*random.sample(range(255), 3))
#rListWidget.setStyleSheet("background-color: {}".format(color.name()))
        