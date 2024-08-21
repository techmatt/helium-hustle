
from __future__ import annotations

from PyQt6.QtWidgets import QDialog, QVBoxLayout, QLabel, QPushButton
from PyQt6.QtCore import Qt

from gameDatabase import EventInfo

class EventManager():
    def __init__(self, state : GameState):
        self.state = state
        self.events = []
        
    def eventShouldTrigger(self, eState : EventState) -> bool:
        eInfo : EventInfo = eState.info
        if self.state.ticks < eInfo.ticksRequired:
            return False
        
    def triggerEvent(self, eState : EventState):
        eState.triggered = True
        
    def step(self):
        for eState in self.state.events.values():
            #print(eState)
            #print(dir(eState))
            if eState.triggered:
                continue
            if self.eventShouldTrigger(eState):
                self.triggerEvent(eState)
                continue

