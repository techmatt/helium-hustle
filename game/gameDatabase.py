
from __future__ import annotations

import json
import os
from typing import Dict, List, NamedTuple

class CommandInfo(NamedTuple):
    name: str
    production: Dict[str, float]
    cost: Dict[str, float]
    startUnlocked: bool
    description: str
    
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
    upkeep: Dict[str, float]
    storage: Dict[str, float]
    canDeactivate: bool
    description: str

class GameParams:
    def __init__(self, database : GameDatabase):
        
        self.startingStorage: Dict[str, float] = {}
        for rName in database.resources.keys():
            self.startingStorage[rName] = 0.0
            
        self.startingResources: Dict[str, float] = {}
        for rName in database.resources.keys():
            self.startingResources[rName] = 0.0
            
        self.startingBuildings: Dict[str, int] = {}
        for bName in database.buildings.keys():
            self.startingBuildings[bName] = 0
        
        self.startingStorage["Credits"] = 1000.0
        self.startingStorage["Electricity"] = 100.0
        self.startingStorage["Processors"] = 1
        self.startingStorage["Land"] = 100
        
        self.startingResources["Credits"] = 500.0
        self.startingResources["Land"] = self.startingStorage["Land"]
        
        self.startingBuildings["Solar Panels"] = 1
        self.startingBuildings["Storage Facility"] = 1
        
        self.timerInterval = 1000 # timer interval in milliseconds
        
        self.maxProgramCount = 5

class GameDatabase:
    def __init__(self, filePath):
        with open(filePath, 'r') as file:
            data = json.load(file)

        self.commands: Dict[str, CommandInfo] = {}
        for r in data['commands']:
            curCommand = CommandInfo(
                name = r['name'],
                production = r['production'],
                cost = r['cost'],
                startUnlocked = r['startUnlocked'],
                description = r['description']
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
                upkeep = b.get('upkeep', {}),
                storage = b.get('storage', {}),
                costScaling = b['costScaling'],
                canDeactivate = b['canDeactivate'],
                description = b['description'])
            self.buildings[curBuilding.name] = curBuilding
            
        self.params: GameParams = GameParams(self)

    def saveToJSON(self, filePath: str):
        data = {
            "commands": [
                {
                    "name": c.name,
                    "production": c.production,
                    "costs": c.costs,
                    "startUnlocked": c.startUnlocked,
                    "description": c.description
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
                    "upkeep": b.maintenance,
                    "storage": b.storage,
                    "costScaling": b.costScaling,
                    "canDeactivate": b.canDeactivate,
                    "description": b.description
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