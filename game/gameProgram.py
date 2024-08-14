
from __future__ import annotations

import json
import os
import math
from typing import Dict, List, NamedTuple

from gameDatabase import GameDatabase, CommandInfo, ResourceInfo, BuildingInfo

class GameCommand:
    def __init__(self, info : CommandInfo):
        self.info: CommandInfo = info
        self.count = 0
        self.maxCount = 1

class GameProgram:
    def __init__(self, state : GameState):
        self.state: GameState = state
        self.commands: List[GameCommand] = []
        self.instructionPointer: int = 0
        self.assignedProcessors: int = 0

    def step(self):
        if len(self.commands) == 0:
            return
        if self.instructionPointer >= len(self.commands):
            self.instructionPointer = 0
            
        curCommand = self.commands[self.instructionPointer]
        self.state.runCommand(curCommand.info.name)
        curCommand.count += 1
        if curCommand.count >= curCommand.maxCount:
           curCommand.count.count = 0
           self.instructionPointer += 1