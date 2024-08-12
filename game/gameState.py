
from __future__ import annotations

import json
import os
import math
from typing import Dict, List, NamedTuple

from gameDatabase import GameDatabase, ResourceInfo, BuildingInfo

class CommandState:
    def __init__(self, info : CommandInfo):
        self.info: CommandInfo = info
        self.unlocked: bool = info.startUnlocked

class BuildingState:
    def __init__(self, info : BuildingInfo):
        self.info: BuildingInfo = info
        self.count: int = 0
        
class ResourceState:
    def __init__(self, info : ResourceInfo):
        self.info: ResourceInfo = info
        self.income: float = 0
        self.storage: float = 0
        self.count: float = 0

class CostTotal:
    def __init__(self, costs : Dict[str, float]):
        self.costs = costs

class GameState:
    def __init__(self, database : GameDatabase):
        self.database: GameDatabase = database
        
        self.commands: Dict[str, CommandState] = {}
        for c in database.commands.values():
            self.commands[c.name] = CommandState(c)
            
        self.buildings: Dict[str, BuildingState] = {}
        for b in database.buildings.values():
            self.buildings[b.name] = BuildingState(b)

        self.resources: Dict[str, ResourceState] = {}
        for rName, rInfo in database.resources.items():
            rState = ResourceState(rInfo)
            rState.count = database.params.startingResources[rName]
            self.resources[rName] = rState
        
        self.updateAttributes()

    def updateAttributes(self):
        for r in self.resources.values():
            r.income = 0
            r.storage = self.database.params.baseStorage[r.info.name]

        for b in self.buildings.values():
            for rName, prod in b.info.production.items():
                rState = self.resources[rName]
                rState.income += b.count * prod

            for rName, stor in b.info.storage.items():
                rState = self.resources[rName]
                rState.storage += b.count * stor

    def step(self):
        for r in self.resources.values():
            r.count = min(r.count + r.income, r.storage)
    
    def getBuildingCost(self, buildingName : str) -> CostTotal:
        b = self.buildings[buildingName]
        
        costs: Dict[str, float] = {}
        costMultiplier = pow(b.info.costScaling, b.count)
        for rName, baseCost in b.info.baseCost.items():
            costs[rName] = math.floor(baseCost * costMultiplier)
            
        return CostTotal(costs)
    
    def canAffordCost(self, costTotal : CostTotal) -> bool:
        for r, v in costTotal.costs.items():
            if v > self.resources[r].count:
                return False
        return True

    def spendCost(self, costTotal : CostTotal):
        for r, v in costTotal.costs.items():
            if v > self.resources[r].count:
                print('cannot afford cost')
                return
            self.resources[r].count -= v

    def attemptPurchaseBuilding(self, buildingName):
        buildingCost = self.getBuildingCost(buildingName)
        if not self.canAffordCost(buildingCost):
            print('cannot afford ' + buildingName)
            return
        
        self.spendCost(buildingCost)
        self.buildings[buildingName].count += 1
        self.updateAttributes()
        
    def runCommand(self, commandName):
        cState : CommandState = self.commands[commandName]
        cInfo = cState.info
        
        for rName, v in cInfo.production.items():
            r = self.resources[rName]
            r.count = min(r.count + v, r.storage)
            
        self.updateAttributes()

if __name__ == "__main__":
    print('testing game state')
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    for i in range(0, 10):
        state.step()