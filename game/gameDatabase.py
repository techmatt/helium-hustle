
from __future__ import annotations

import json
import os
from typing import Dict, List, NamedTuple

class CommandInfo(NamedTuple):
    name: str
    production: Dict[str, float]
    cost: Dict[str, float]
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

class EventInfo(NamedTuple):
    name: str
    resourcesRequired: Dict[str, float]
    buildingsRequired: Dict[str, int]
    ticksRequired: int
    unlocks: List[str]
    income: Dict[str, float]
    flavorText: str
    mechanicsText: str
    
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
        
        self.startingUnlocks: List[str] = [
            "Solar Panels",
            "Storage Facility",
            "Credits",
            "Land",
            "Electricity",
            "Processors"
            ]
        
        self.timerInterval = 1000 # timer interval in milliseconds
        
        self.maxProgramCount = 5

class GameDatabase:
    def __init__(self, filePathBase):

        with open(filePathBase + 'Buildings.json', 'r') as file:
            buildingData = json.load(file)

        with open(filePathBase + 'Commands.json', 'r') as file:
            commandData = json.load(file)            
            
        with open(filePathBase + 'Resources.json', 'r') as file:
            resourceData = json.load(file)
            
        with open(filePathBase + 'Events.json', 'r') as file:
            eventData = json.load(file)

        self.commands: Dict[str, CommandInfo] = {}
        for c in commandData['commands']:
            curCommand = CommandInfo(
                name = c['name'],
                production = c['production'],
                cost = c['cost'],
                description = c['description']
            )
            self.commands[curCommand.name] = curCommand
            
        self.resources: Dict[str, ResourceInfo] = {}
        for r in resourceData['resources']:
            curResource = ResourceInfo(
                name = r['name'],
                basePrice = r['basePrice'],
                baseDemand = r['baseDemand'],
                elasticity = r['elasticity']
            )
            self.resources[curResource.name] = curResource

        self.buildings: Dict[str, BuildingInfo] = {}
        for b in buildingData['buildings']:
            curBuilding = BuildingInfo(
                name = b['name'],
                baseCost = b.get('baseCost', {}),
                production = b.get('production', {}),
                upkeep = b.get('upkeep', {}),
                storage = b.get('storage', {}),
                costScaling = b['costScaling'],
                canDeactivate = b['canDeactivate'],
                description = b['description']
            )
            self.buildings[curBuilding.name] = curBuilding
        
        self.events: Dict[str, EventInfo] = {}
        for e in eventData['events']:
            curEvent = EventInfo(
                name = e['name'],
                resourcesRequired = e['resourcesRequired'],
                buildingsRequired = e['buildingsRequired'],
                ticksRequired = e['ticksRequired'],
                unlocks = e['unlocks'],
                income = e['income'],
                flavorText = e['flavorText'],
                mechanicsText = e['mechanicsText']
            )
            self.events[curEvent.name] = curEvent
            
        self.params: GameParams = GameParams(self)

    def saveToJSON(self, filePath: str):
        data = {
            "commands": [
                {
                    "name": c.name,
                    "production": c.production,
                    "costs": c.costs,
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
            ],
            "events": [
                {
                    "name": e.name,
                    "resourcesRequired": e.resourcesRequired,
                    "buildingsRequired": e.buildingsRequired,
                    "ticksRequired": r.ticksRequired,
                    "unlocks": r.unlocks,
                    "income": r.income,
                    "flavorText": r.flavorText,
                    "mechanicsText": r.mechanicsText
                } for e in self.events.values()
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