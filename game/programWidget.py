

from __future__ import annotations

from PyQt6.QtWidgets import (QWidget, QListWidget, QHBoxLayout, QVBoxLayout,
                             QLabel, QProgressBar, QPushButton, QListWidgetItem)
from PyQt6.QtCore import Qt
from gameProgram import GameProgram, GameCommand

class ProgramItemWidget(QWidget):
    def __init__(self, command : GameCommand):
        super().__init__()
        layout = QHBoxLayout(self)
        self.label = QLabel(command.info.name)
        self.progressBar = QProgressBar()
        self.progressBar.setValue(1)
        self.removeButton = QPushButton("help")

        layout.addWidget(self.label)
        layout.addWidget(self.progressBar)
        layout.addWidget(self.removeButton)

class ProgramWidget(QWidget):
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.layout = QVBoxLayout(self)
        self.listWidget = QListWidget()
        self.layout.addWidget(self.listWidget)

        # Enable drag and drop
        self.listWidget.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        self.listWidget.setMovement(QListWidget.Movement.Free)
        self.listWidget.setDefaultDropAction(Qt.DropAction.MoveAction)
        self.listWidget.setDragEnabled(True)

        state : GameState = gameUI.state
        activeProgram : GameProgram = state.programs[gameUI.visibleProgramIndex]

        for command in activeProgram.commands:
            self.addCommand(command)

        #self.add_item("Balance Your Chi", 1, 100)
        #self.add_item("Recruiting Followers", 1, 100)
        #self.add_item("Communing With Divinity", 2, 50)
        #self.add_item("Mind Cultivation", 1, 100)
        #self.add_item("Communing With Divinity", 1, 100)
        #self.add_item("Body Cultivation", 1, 100)

    def addCommand(self, command : GameCommand):
        item = QListWidgetItem(self.listWidget)
        itemWidget = ProgramItemWidget(command)
        item.setSizeHint(itemWidget.sizeHint())
        self.listWidget.addItem(item)
        self.listWidget.setItemWidget(item, itemWidget)

        # Connect buttons
        #item_widget.up_button.clicked.connect(lambda: self.move_item(item, -1))
        #item_widget.down_button.clicked.connect(lambda: self.move_item(item, 1))
        itemWidget.removeButton.clicked.connect(lambda: self.removeItem(item))

    def move_item(self, item, direction):
        current_row = self.listWidget.row(item)
        new_row = current_row + direction
        if 0 <= new_row < self.listWidget.count():
            taken_item = self.listWidget.takeItem(current_row)
            self.listWidget.insertItem(new_row, taken_item)
            self.listWidget.setItemWidget(taken_item, self.listWidget.itemWidget(item))

    def removeItem(self, item):
        self.listWidget.takeItem(self.listWidget.row(item))
