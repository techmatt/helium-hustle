

from __future__ import annotations

from math import floor

from PyQt6.QtWidgets import (QWidget, QListWidget, QHBoxLayout, QVBoxLayout,
                             QLabel, QProgressBar, QPushButton, QListWidgetItem)
from PyQt6.QtCore import Qt

from gameProgram import GameProgram, GameCommand

class ProgramItemWidget(QWidget):
    def __init__(self, command : GameCommand):
        super().__init__()
        
        self.command = command
        
        layout = QHBoxLayout(self)
        self.label = QLabel(command.info.name)
        self.progressBar = QProgressBar()
        self.progressBar.setRange(0, 1000)
        self.progressBar.setValue(0)
        self.progressBar.setTextVisible(False)
        self.subButton = QPushButton("-")
        self.addButton = QPushButton("+")
        self.removeButton = QPushButton("X")

        layout.addWidget(self.label)
        layout.addWidget(self.progressBar)
        layout.addWidget(self.subButton)
        layout.addWidget(self.addButton)
        layout.addWidget(self.removeButton)

class ProgramWidget(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.layout = QVBoxLayout(self)
        self.listWidget = QListWidget()
        self.layout.addWidget(self.listWidget)
        self.gameUI = gameUI

        # Enable drag and drop
        self.listWidget.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        self.listWidget.setMovement(QListWidget.Movement.Free)
        self.listWidget.setDefaultDropAction(Qt.DropAction.MoveAction)
        self.listWidget.setDragEnabled(True)
        self.listWidget.model().rowsMoved.connect(self.onRowsMoved)

        state : GameState = gameUI.state
        activeProgram : GameProgram = state.programs[gameUI.visibleProgramIndex]

        for command in activeProgram.commands:
            self.addCommand(command)

    def onRowsMoved(self, parent, start, end, destination, row):
        self.loadVisibleProgramFromList()
        
    def loadVisibleProgramFromList(self):
        state : GameState = self.gameUI.state
        activeProgram : GameProgram = state.programs[self.gameUI.visibleProgramIndex]
        activeProgram.commands = []
        for i in range(self.listWidget.count()):
            item = self.listWidget.item(i)
            widget = self.listWidget.itemWidget(item)
            activeProgram.commands.append(widget.command)
        activeProgram.resetAllCommands()
        
    def updateProgressBars(self):
        state : GameState = self.gameUI.state
        activeProgram : GameProgram = state.programs[self.gameUI.visibleProgramIndex]
        for i in range(self.listWidget.count()):
            if i >= len(activeProgram.commands):
                continue
            c = activeProgram.commands[i]
            
            item = self.listWidget.item(i)
            widget = self.listWidget.itemWidget(item)
            
            targetValue = floor(c.count * 1000 / c.maxCount)
            delta = abs(targetValue - widget.progressBar.value())
            if delta > 0.00001:
                widget.progressBar.setValue(targetValue)
            
    def addCommand(self, command : GameCommand):
        item = QListWidgetItem(self.listWidget)
        itemWidget = ProgramItemWidget(command)
        item.setSizeHint(itemWidget.sizeHint())
        self.listWidget.addItem(item)
        self.listWidget.setItemWidget(item, itemWidget)

        # Connect buttons
        itemWidget.addButton.clicked.connect(lambda: self.freqClick(item, 1))
        itemWidget.subButton.clicked.connect(lambda: self.freqClick(item, -1))
        itemWidget.removeButton.clicked.connect(lambda: self.removeItem(item))

    def freqClick(self, item, direction):
        state : GameState = self.gameUI.state
        activeProgram : GameProgram = state.programs[self.gameUI.visibleProgramIndex]
        activeCommand = activeProgram.commands[self.listWidget.row(item)]
        activeCommand.maxCount += direction
        activeCommand.maxCount = min(activeCommand.maxCount, 1000)
        activeCommand.maxCount = max(activeCommand.maxCount, 1)
        activeCommand.count = 0
        
    def removeItem(self, item):
        self.listWidget.takeItem(self.listWidget.row(item))
        self.loadVisibleProgramFromList()

#def move_item(self, item, direction):
#        current_row = self.listWidget.row(item)
#        new_row = current_row + direction
#       if 0 <= new_row < self.listWidget.count():
#           taken_item = self.listWidget.takeItem(current_row)
#            self.listWidget.insertItem(new_row, taken_item)
#            self.listWidget.setItemWidget(taken_item, self.listWidget.itemWidget(item))
