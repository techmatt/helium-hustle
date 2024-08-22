
import os
import sys

# Add the parent directory to sys.path. this is a simpler alternative to using setup.py
# and requiring everyone add this as a package.
currentDir = os.path.dirname(os.path.abspath(__file__))
parentDir = os.path.dirname(currentDir)
sys.path.append(parentDir)

from PyQt6.QtWidgets import QApplication

from game.database.gameDatabase import GameDatabase
from gameState import GameState
from gameUI import GameUI

if __name__ == '__main__':
    print('starting game')
    
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameData')
    state = GameState(database)
    
    app = QApplication(sys.argv)
    game = GameUI(state, database)
    game.show()
    sys.exit(app.exec())