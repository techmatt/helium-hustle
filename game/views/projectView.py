
from __future__ import annotations

import random
import math
from functools import partial

from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout, QSizePolicy, QProgressBar
from PyQt6.QtGui import QPixmap, QFont, QIcon, QPainter, QColor
from PyQt6.QtCore import Qt, pyqtSignal, QTimer, QSize, QCoreApplication

from game.database.gameDatabase import BuildingInfo, GameDatabase
from game.core.gameState import BuildingState, GameState
from game.ui.collapsibleMenuWidget import CollapsibleMenuWidget, CollapsibleSectionEntries
from game.util.styleSheets import StyleSheets

class ProjectProgressBar(QProgressBar):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setTextVisible(True)
        self.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.setStyleSheet(StyleSheets.GENERAL_12PT_BOLD)

    def setValue(self, value):
        super().setValue(math.floor(value))
        self.updateText()

    def setMaximum(self, maximum):
        super().setMaximum(math.floor(maximum))
        self.updateText()

    def updateText(self):
        current = self.value()
        maximum = self.maximum()
        self.setFormat(f"{current} / {maximum}")

    def setValueAndMaximum(self, value, maximum):
        self.setMaximum(maximum)
        self.setValue(value)
        
class ProjectButtonWidget(QPushButton):
    clicked = pyqtSignal(str)

    def __init__(self, gameUI : GameUI, pName : str):
        super().__init__()
        self.pName = pName
        self.gameUI = gameUI
        
        self.setFixedWidth(270)
        self.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.MinimumExpanding)
        
        state = gameUI.state
        pState = state.projects[pName]
        pInfo : ProjectInfo = pState.info
        
        layout = QVBoxLayout(self)
        layout.setSpacing(2)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        titleWidget = self.makeTitleWidget()
        
        self.progressBar = ProjectProgressBar()
        
        projectIconW, projectIconH = 60, 264
        projectIconLabel = gameUI.makeIconLabel('icons/projects/' + pName + '.png', projectIconH, projectIconW)
        projectIconLabel.setAlignment(Qt.AlignmentFlag.AlignCenter | Qt.AlignmentFlag.AlignTop)
        
        rListWidget = QWidget()
        rListLayout = QVBoxLayout(rListWidget)
        rListLayout.setSpacing(0)
        rListLayout.setContentsMargins(0, 0, 0, 0)
        #rListLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        
        self.rPaymentLabels = {}
        
        for rName, rate in pInfo.resourceRates.items():
            rWidget = self.makeResourceRowWidget(rName, rate)
            rListLayout.addWidget(rWidget)
        #rListLayout.addStretch()

        
        descWidget = QLabel(pInfo.description)
        descWidget.setStyleSheet(StyleSheets.BUILDING_DESCRIPTION)
        descWidget.setWordWrap(True)
        
        #layout.addWidget(editButtonsWidget)
        layout.addWidget(titleWidget)
        layout.addWidget(self.progressBar)
        layout.addWidget(projectIconLabel)
        layout.addWidget(rListWidget)
        layout.addWidget(descWidget)
        
        layout.addStretch(0)
        
        self.updateLabels()
        
    def makeResourceRowWidget(self, rName : str, rate : float):
        rWidget = QWidget()
        rLayout = QGridLayout(rWidget)
        rLayout.setSpacing(0)
        rLayout.setContentsMargins(0, 0, 0, 0)
        rLayout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
            
        rIconLabel = self.gameUI.makeIconLabel('icons/resources/' + rName + '.png', 20, 20)
        rIconLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        
        rNameLabel  = QLabel(f"{rName} (x{rate})")
        
        rPaymentLabel = QLabel("X/s")
        rNameLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)
        rPaymentLabel.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)
        rNameLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)
        rPaymentLabel.setStyleSheet(StyleSheets.BUILDING_RESOURCE_LIST)

        self.rPaymentLabels[rName] = rPaymentLabel
        
        rPaymentWidget = QWidget()
        rPaymentLayout = QHBoxLayout(rPaymentWidget)
        
        subButton = QPushButton("-")
        addButton = QPushButton("+")
        
        subButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        addButton.setStyleSheet(StyleSheets.BUILDING_TITLE)
        
        buttonSize = QSize(22, 22)
        subButton.setFixedSize(buttonSize)
        addButton.setFixedSize(buttonSize)

        addValue = 1 / self.gameUI.state.database.params.ticksPerPlayerSecond
        subValue = -addValue

        subButton.clicked.connect(partial(self.gameUI.modifyProjectPayment, self.pName, rName, subValue))
        addButton.clicked.connect(partial(self.gameUI.modifyProjectPayment, self.pName, rName, addValue))
            
        rPaymentLayout.addWidget(rPaymentLabel)
        rPaymentLayout.addWidget(subButton)
        rPaymentLayout.addWidget(addButton)
        
        rLayout.addWidget(rIconLabel, 0, 0)
        rLayout.addWidget(rNameLabel, 0, 1)
        rLayout.addWidget(rPaymentWidget, 0, 2)
        rLayout.setColumnStretch(0, 0)
        rLayout.setColumnStretch(1, 3)
        rLayout.setColumnStretch(2, 3)
        return rWidget
        
    def makeTitleWidget(self):
        pState = self.gameUI.state.projects[self.pName]
        if pState.info.repeatable:
            nameLabel = QLabel(f"{self.pName} {pState.purchaseCount + 1}")
        else:
            nameLabel = QLabel(f"{self.pName}")
        nameLabel.setAlignment(Qt.AlignmentFlag.AlignLeft)
        nameLabel.setStyleSheet(StyleSheets.BUILDING_TITLE)
        return nameLabel
    
    def updateLabels(self):
        state = self.gameUI.state
        pState = state.projects[self.pName]
        projectCost = state.getProjectCost(self.pName)
        self.progressBar.setValueAndMaximum(pState.progress, projectCost)
           
        state : GameState = self.gameUI.state
        pState : ProjectState = state.projects[self.pName]
        
        for rName, rPayment in pState.resourcePayments.items():
            displayNum = round(-rPayment * state.database.params.ticksPerPlayerSecond, 1)
            self.rPaymentLabels[rName].setText(f"{displayNum}".rstrip('0').rstrip('.') + "/s")
                
        self.update()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        if self.underMouse():
            painter.setBrush(QColor(150, 200, 150, 100))
        else:
            painter.setBrush(QColor(180, 230, 180, 100))
    
        painter.setPen(QColor(180, 180, 180))
        painter.drawRoundedRect(self.rect(), 10, 10)

    def enterEvent(self, event):
        self.update()

    def leaveEvent(self, event):
        self.update()
        
class ProjectView():
    def __init__(self, gameUI : GameUI):
        super().__init__()
        self.gameUI = gameUI
        
        self.sections: Dict[str, CollapsibleSectionEntries] = {}

        for projectCategory in self.gameUI.state.database.params.projectCategories:
            self.sections[projectCategory] = CollapsibleSectionEntries(projectCategory)
        
        for pState in self.gameUI.state.projects.values():
            entryWidget = ProjectButtonWidget(self.gameUI, pState.info.name)
            #entryWidget.clicked.connect(gameUI.purchaseBuilding)
            self.sections[pState.info.category].childWidgets.append(entryWidget)

        self.mainWidget = CollapsibleMenuWidget(gameUI, self.sections, "grid")
        
    def updateLabels(self):
        self.mainWidget.updateLabels()


        
    