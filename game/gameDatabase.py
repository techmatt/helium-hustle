
from __future__ import annotations

import json
import os
from typing import Dict, List, NamedTuple
    
class CommandInfo(NamedTuple):
    name: str
    production: Dict[str, float]
    startUnlocked: bool
    
class ResourceInfo(NamedTuple):
    name: str
    basePrice: float
    baseDemand: float
    elasticity: float

class BuildingInfo(NamedTuple):
    name: str
    costScaling: float
    baseCost: Dict[str, float]
    production: Dict[str, float]
    storage: Dict[str, float]

class GameParams:
    def __init__(self, database : GameDatabase):
        
        self.baseStorage: Dict[str, float] = {}
        for rName in database.resources.keys():
            self.baseStorage[rName] = 0.0
            
        self.baseStorage["Credits"] = 1000.0
        
        self.startingResources: Dict[str, float] = {}
        for rName in database.resources.keys():
            self.startingResources[rName] = 0.0
            
        self.startingResources["Credits"] = 500.0
        
        self.timerInterval = 500 # timer interval in milliseconds

class GameDatabase:
    def __init__(self, filePath):
        with open(filePath, 'r') as file:
            data = json.load(file)

        self.commands: Dict[str, CommandInfo] = {}
        for r in data['commands']:
            curCommand = CommandInfo(
                name = r['name'],
                production = r['production'],
                startUnlocked = r['startUnlocked']
            )
            self.commands[curCommand.name] = curCommand
            
        self.resources: Dict[str, ResourceInfo] = {}
        for r in data['resources']:
            curResource = ResourceInfo(
                name = r['name'],
                basePrice = r['basePrice'],
                baseDemand = r['baseDemand'],
                elasticity = r['elasticity']
            )
            self.resources[curResource.name] = curResource

        self.buildings: Dict[str, BuildingInfo] = {}
        for b in data['buildings']:
            curBuilding = BuildingInfo(
                name = b['name'],
                baseCost = b.get('baseCost', {}),
                production = b.get('production', {}),
                storage = b.get('storage', {}),
                costScaling = b['costScaling'])
            self.buildings[curBuilding.name] = curBuilding
            
        self.params: GameParams = GameParams(self)

    def saveToJSON(self, filePath: str):
        data = {
            "commands": [
                {
                    "name": c.name,
                    "production": c.production,
                    "startUnlocked": c.startUnlocked
                } for c in self.commands.values()
            ],
            "resources": [
                {
                    "name": r.name,
                    "basePrice": r.basePrice,
                    "baseDemand": r.baseDemand,
                    "elasticity": r.elasticity
                } for r in self.resources.values()
            ],
            "buildings": [
                {
                    "name": b.name,
                    "baseCost": b.baseCost,
                    "production": b.production,
                    "storage": b.storage
                } for b in self.buildings.values()
            ]
        }

        with open(filePath, 'w') as file:
            json.dump(data, file, indent=2)

if __name__ == "__main__":
    print('testing game database')
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    database.saveToJSON('gameDatabaseEcho.json')
    print('game database saved to gameDatabaseEcho.json')       