
from __future__ import annotations

import json
import os
from typing import Dict, List, NamedTuple

from game.util.enums import Ideology

class CommandInfo(NamedTuple):
    name: str
    production: Dict[str, float]
    cost: Dict[str, float]
    category: str
    description: str
    
class BuildingInfo(NamedTuple):
    name: str
    costScaling: float
    baseCost: Dict[str, float]
    production: Dict[str, float]
    upkeep: Dict[str, float]
    storage: Dict[str, float]
    canDeactivate: bool
    category: str
    description: str

class ResearchInfo(NamedTuple):
    name: str
    cost: Dict[str, float]
    ideology: Ideology
    unlocks: List[str]
    category: str
    description: str
    
class ProjectInfo(NamedTuple):
    name: str
    baseCost: float
    repeatable: bool
    costScaling: float # only relevant for repeatable projects
    resourceRates: Dict[str, float]
    persistentR: bool # project lasts between retirements
    persistentT: bool # project lasts between time travel
    unlocks: List[str]
    ideology: Ideology
    category: str
    description: str

class ResourceInfo(NamedTuple):
    name: str
    basePrice: float
    baseDemand: float
    elasticity: float

class EventInfo(NamedTuple):
    name: str
    resourcesRequired: Dict[str, float]
    buildingsRequired: Dict[str, int]
    ticksRequired: int
    unlocks: List[str]
    income: Dict[str, float]
    flavorText: str
    mechanicsText: str
    options: []

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
        self.startingStorage["Energy"] = 100.0
        self.startingStorage["Boredom"] = 1000.0
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
            "Energy",
            "Processors"
            ]
        
        self.timerInterval = 250 # timer interval in milliseconds
        self.intervalsPerSecond = 4.0

        self.gameSecondsPerTick = 60 # each tick is a game minute
        self.ticksPerProcessorCycle = 4 # the processors operate more slowly than the game clock
        
        self.maxProgramCount = 5
        
        self.commandCategories = ["Computation", "Manual Operation", "Science"]
        self.researchCategories = ["Production", "Programming", "Defensive", "Offensive"]
        self.projectCategories = ["Robot Welfare", "Temporal Constructs"]
        self.buildingCategories = ["Mining", "Power", "Storage", "Processors", "Economy"]

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
            
        with open(filePathBase + 'Research.json', 'r') as file:
            researchData = json.load(file)
            
        with open(filePathBase + 'Projects.json', 'r') as file:
            projectData = json.load(file)

        self.commands: Dict[str, CommandInfo] = {}
        for c in commandData['commands']:
            curCommand = CommandInfo(
                name = c['name'],
                production = c['production'],
                cost = c['cost'],
                category = c['category'],
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
                canDeactivate = b.get('canDeactivate', False),
                category = b['category'],
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
                mechanicsText = e['mechanicsText'],
                options = e.get('options', [])
            )
            self.events[curEvent.name] = curEvent
            
        self.research: Dict[str, ResearchInfo] = {}
        for r in researchData['research']:
            curResearch = ResearchInfo(
                name = r['name'],
                cost = r['cost'],
                ideology = Ideology[r['ideology']],  # Convert string to Ideology enum
                category = r['category'],
                unlocks = r['unlocks'],
                description = r['description']
            )
            self.research[curResearch.name] = curResearch

        self.projects: Dict[str, ProjectInfo] = {}
        for p in projectData['projects']:
            curProject = ProjectInfo(
                name = p['name'],
                baseCost = float(p['baseCost']),
                repeatable = bool(p['repeatable']),
                costScaling = float(p['costScaling']),
                resourceRates = {k: float(v) for k, v in p['resourceRates'].items()},
                persistentR = bool(p['persistentR']),
                persistentT = bool(p['persistentT']),
                unlocks = p['unlocks'],
                ideology = Ideology[p['ideology']],
                category = p['category'],
                description = p['description']
            )
            self.projects[curProject.name] = curProject
            
        self.params: GameParams = GameParams(self)
        self.verifyData()
        
    def verifyData(self):
        for b in self.buildings.values():
            if not (b.category in self.params.buildingCategories):
                raise ValueError(f"Invalid building category {b.category}")
            
        for r in self.research.values():
            if not (r.category in self.params.researchCategories):
                raise ValueError(f"Invalid research category {r.category}")
            
        for p in self.projects.values():
            if not (p.category in self.params.projectCategories):
                raise ValueError(f"Invalid project category {p.category}")

        for c in self.commands.values():
            if not (c.category in self.params.commandCategories):
                raise ValueError(f"Invalid command category {c.category}")
        
        
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