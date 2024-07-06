
import sys
import os
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel

from gameDatabase import GameDatabase
from gameState import GameState

class GameUI(QMainWindow):
    def __init__(self, state):
        super().__init__()
        self.setWindowTitle("Helium Hustle")
        self.setGeometry(100, 100, 600, 400)

        self.state = state
        
        self.cookies = 0
        
        self.init_ui()

    def init_ui(self):
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QHBoxLayout(central_widget)

        # Left frame (menu)
        left_frame = QWidget()
        left_layout = QVBoxLayout(left_frame)
        main_game_btn = QPushButton("Main Game")
        upgrades_btn = QPushButton("Upgrades")
        left_layout.addWidget(main_game_btn)
        left_layout.addWidget(upgrades_btn)
        left_layout.addStretch()

        # Right frame (content)
        self.right_frame = QWidget()
        self.right_layout = QVBoxLayout(self.right_frame)
        self.cookies_label = QLabel(f"Cookies: {self.cookies}")
        self.click_button = QPushButton("Click me!")
        self.right_layout.addWidget(self.cookies_label)
        self.right_layout.addWidget(self.click_button)

        main_layout.addWidget(left_frame, 1)
        main_layout.addWidget(self.right_frame, 3)

        # Connect signals
        self.click_button.clicked.connect(self.click_cookie)
        main_game_btn.clicked.connect(self.show_main_game)
        upgrades_btn.clicked.connect(self.show_upgrades)

    def click_cookie(self):
        self.cookies += self.cookies_per_click
        self.update_cookies_label()

    def update_cookies_label(self):
        self.cookies_label.setText(f"Cookies: {self.cookies}\nPer click: {self.cookies_per_click}\nPer second: {self.cookies_per_second}")

    def show_main_game(self):
        # Clear right layout and add main game widgets
        self.clear_right_layout()
        self.right_layout.addWidget(self.cookies_label)
        self.right_layout.addWidget(self.click_button)

    def show_upgrades(self):
        # Clear right layout and add upgrade widgets
        self.clear_right_layout()
        upgrade_click_btn = QPushButton("Upgrade Click (+1)")
        upgrade_passive_btn = QPushButton("Upgrade Passive (+1/s)")
        self.right_layout.addWidget(upgrade_click_btn)
        self.right_layout.addWidget(upgrade_passive_btn)
        upgrade_click_btn.clicked.connect(lambda: self.upgrade("click"))
        upgrade_passive_btn.clicked.connect(lambda: self.upgrade("passive"))

    def clear_right_layout(self):
        while self.right_layout.count():
            item = self.right_layout.takeAt(0)
            widget = item.widget()
            if widget:
                widget.setParent(None)

    def upgrade(self, upgrade_type):
        if upgrade_type == "click" and self.cookies >= 10:
            self.cookies -= 10
            self.cookies_per_click += 1
        elif upgrade_type == "passive" and self.cookies >= 20:
            self.cookies -= 20
            self.cookies_per_second += 1
        self.update_cookies_label()

if __name__ == '__main__':
    print('starting UI')
    
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    
    app = QApplication(sys.argv)
    game = GameUI(state)
    game.show()
    sys.exit(app.exec())