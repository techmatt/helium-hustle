
import json
import os
from typing import Dict, List, NamedTuple

from gameDatabase import GameDatabase, ResourceInfo, BuildingInfo

class BuildingState:
    def __init__(self, info : BuildingInfo):
        self.info = info
        self.count: int = 0

class ResourceState:
    def __init__(self, info : ResourceInfo):
        self.info = info
        self.income: float = 0
        self.storage: float = 0
        self.count: float = 0

class GameState:
    def __init__(self, database):
        self.database: GameDatabase = database
        
        self.buildings: Dict[str, BuildingState] = {}
        for b in database.buildings.values():
            self.buildings[b.name] = BuildingState(b)

        self.resources: Dict[str, ResourceState] = {}
        for r in database.resources.values():
            self.resources[r.name] = ResourceState(r)

    def updateAttributes(self):
        for r in self.resources.values():
            r.income = 0
            r.storage = 0

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

class CostTotal:
    def __init__(self, costs : Dict[str, float]):
        self.costs = costs

    def canAfford(self, state : GameState) -> bool:
        for r, v in self.costs.items():
            if self.costs[r] > state.resources[r].count:
                return False
        return True
    
if __name__ == "__main__":
    print('testing game state')
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    for i in range(0, 10):
        state.step()