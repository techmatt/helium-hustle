

from PyQt6.QtWidgets import (QWidget, QListWidget, QHBoxLayout, QVBoxLayout,
                             QLabel, QProgressBar, QPushButton, QListWidgetItem)
from PyQt6.QtCore import Qt

class ProgramItemWidget(QWidget):
    def __init__(self, text, days, progress):
        super().__init__()
        layout = QHBoxLayout(self)
        self.label = QLabel(f"{text} ({days} days)")
        self.progressBar = QProgressBar()
        self.progressBar.setValue(progress)
        self.remove_button = QPushButton("help")

        layout.addWidget(self.label)
        layout.addWidget(self.progressBar)
        layout.addWidget(self.remove_button)

class ProgramWidget(QWidget):
    def __init__(self):
        super().__init__()
        self.layout = QVBoxLayout(self)
        self.list_widget = QListWidget()
        self.layout.addWidget(self.list_widget)

        # Enable drag and drop
        self.list_widget.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        self.list_widget.setMovement(QListWidget.Movement.Free)
        self.list_widget.setDefaultDropAction(Qt.DropAction.MoveAction)
        self.list_widget.setDragEnabled(True)

        # Add some example items
        self.add_item("Balance Your Chi", 1, 100)
        self.add_item("Recruiting Followers", 1, 100)
        self.add_item("Communing With Divinity", 2, 50)
        self.add_item("Mind Cultivation", 1, 100)
        self.add_item("Communing With Divinity", 1, 100)
        self.add_item("Body Cultivation", 1, 100)

    def add_item(self, text, days, progress):
        item = QListWidgetItem(self.list_widget)
        item_widget = ProgramItemWidget(text, days, progress)
        item.setSizeHint(item_widget.sizeHint())
        self.list_widget.addItem(item)
        self.list_widget.setItemWidget(item, item_widget)

        # Connect buttons
        #item_widget.up_button.clicked.connect(lambda: self.move_item(item, -1))
        #item_widget.down_button.clicked.connect(lambda: self.move_item(item, 1))
        item_widget.remove_button.clicked.connect(lambda: self.remove_item(item))

    def move_item(self, item, direction):
        current_row = self.list_widget.row(item)
        new_row = current_row + direction
        if 0 <= new_row < self.list_widget.count():
            taken_item = self.list_widget.takeItem(current_row)
            self.list_widget.insertItem(new_row, taken_item)
            self.list_widget.setItemWidget(taken_item, self.list_widget.itemWidget(item))

    def remove_item(self, item):
        self.list_widget.takeItem(self.list_widget.row(item))
