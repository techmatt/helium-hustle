
import sys
import os
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt6.QtGui import QPixmap, QFont

from gameDatabase import GameDatabase
from gameState import GameState

class BuildingWidget(QWidget):
    def __init__(self, state : GameState, name : str):
        super().__init__()
        
        layout = QVBoxLayout()
        
        bState = state.buildings[name]
        buildingCost = state.getBuildingCost(name)
            
        #buildBtn = QPushButton(f"{b.name}")
        #self.rightLayout.addWidget(buildBtn)
                
        # Name
        nameLabel = QLabel(f"{name} ({bState.count})")
        nameLabel.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        layout.addWidget(nameLabel)
        
        # Cost
        for rName, cost in buildingCost.costs.items():
            costLabel = QLabel(f"{rName} {cost}")
            layout.addWidget(costLabel)
            
        self.setLayout(layout)
        self.setStyleSheet("background-color: #D2B48C; border-radius: 10px; padding: 10px;")

class GameUI(QMainWindow):
    def __init__(self, state : GameState, database : GameDatabase):
        super().__init__()
        self.setWindowTitle("Helium Hustle")
        self.setGeometry(100, 100, 600, 400)

        self.state = state
        self.database = database
        
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
        mainLayout.addWidget(self.rightFrame, 2)

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
        
        buildingGridWidget = QWidget()
        buildingGridLayout = QGridLayout(buildingGridWidget)
        
        for bName in self.database.buildings.keys():
            bWidget = BuildingWidget(self.state, bName)
            buildingGridLayout.addWidget(bWidget)
            
            """buildingCost = self.state.getBuildingCost(b.name)
            
            buildBtn = QPushButton(f"{b.name}")
            self.rightLayout.addWidget(buildBtn)
            
            for rName, cost in buildingCost.costs.items():
                costLabel = QLabel(f"{rName} {cost}")
                self.rightLayout.addWidget(costLabel)"""

        self.rightLayout.addWidget(buildingGridWidget)
        
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
    game = GameUI(state, database)
    game.show()
    sys.exit(app.exec())