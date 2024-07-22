
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
        self.cookiesPerClick = 1
        self.cookiesPerSecond = 1
        
        self.initUI()

    def initUI(self):
        centralWidget = QWidget()
        self.setCentralWidget(centralWidget)
        mainLayout = QHBoxLayout(centralWidget)

        # Left frame (menu)
        leftFrame = QWidget()
        leftLayout = QVBoxLayout(leftFrame)
        mainGameBtn = QPushButton("Main Game")
        upgradesBtn = QPushButton("Upgrades")
        leftLayout.addWidget(mainGameBtn)
        leftLayout.addWidget(upgradesBtn)
        
        self.resourceLabels = []
        for r in self.state.resources.values():
            print('resource: ', r.info.name)
            l = QLabel(f"{r.info.name}: {r.count} / {r.storage} (+{r.income} / sec)")
            leftLayout.addWidget(l)

        leftLayout.addStretch()

        # Right frame (content)
        self.rightFrame = QWidget()
        self.rightLayout = QVBoxLayout(self.rightFrame)
        self.cookiesLabel = QLabel(f"Cookies: {self.cookies}")
        self.clickButton = QPushButton("Click me!")
        self.rightLayout.addWidget(self.cookiesLabel)
        self.rightLayout.addWidget(self.clickButton)

        mainLayout.addWidget(leftFrame, 1)
        mainLayout.addWidget(self.rightFrame, 3)

        # Connect signals
        self.clickButton.clicked.connect(self.clickCookie)
        mainGameBtn.clicked.connect(self.showMainGame)
        upgradesBtn.clicked.connect(self.showUpgrades)

    def clickCookie(self):
        self.cookies += self.cookiesPerClick
        self.updateCookiesLabel()

    def updateCookiesLabel(self):
        self.cookiesLabel.setText(f"Cookies: {self.cookies}\nPer click: {self.cookiesPerClick}\nPer second: {self.cookiesPerSecond}")

    def showMainGame(self):
        # Clear right layout and add main game widgets
        self.clearRightLayout()
        self.rightLayout.addWidget(self.cookiesLabel)
        self.rightLayout.addWidget(self.clickButton)

    def showUpgrades(self):
        # Clear right layout and add upgrade widgets
        self.clearRightLayout()
        upgradeClickBtn = QPushButton("Upgrade Click (+1)")
        upgradePassiveBtn = QPushButton("Upgrade Passive (+1/s)")
        self.rightLayout.addWidget(upgradeClickBtn)
        self.rightLayout.addWidget(upgradePassiveBtn)
        upgradeClickBtn.clicked.connect(lambda: self.upgrade("click"))
        upgradePassiveBtn.clicked.connect(lambda: self.upgrade("passive"))

    def clearRightLayout(self):
        while self.rightLayout.count():
            item = self.rightLayout.takeAt(0)
            widget = item.widget()
            if widget:
                widget.setParent(None)

    def upgrade(self, upgradeType):
        if upgradeType == "click" and self.cookies >= 10:
            self.cookies -= 10
            self.cookiesPerClick += 1
        elif upgradeType == "passive" and self.cookies >= 20:
            self.cookies -= 20
            self.cookiesPerSecond += 1
        self.updateCookiesLabel()

if __name__ == '__main__':
    print('starting UI')
    
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    
    app = QApplication(sys.argv)
    game = GameUI(state)
    game.show()
    sys.exit(app.exec())