from PyQt6.QtWidgets import QDialog, QVBoxLayout, QLabel, QPushButton
from PyQt6.QtCore import Qt

class EventDialog(QDialog):
    def __init__(self, parent, eventText):
        super().__init__(parent)
        
        self.setWindowTitle("Event")
        self.setWindowModality(Qt.WindowModality.ApplicationModal)
        
        layout = QVBoxLayout()
        
        # Event text
        eventLabel = QLabel(eventText)
        eventLabel.setWordWrap(True)
        layout.addWidget(eventLabel)
        
        # OK button
        OKButton = QPushButton("OK")
        OKButton.clicked.connect(self.accept)
        layout.addWidget(OKButton)
        
        self.setLayout(layout)

