
from __future__ import annotations

import json
import os
import math
from typing import Dict, List, NamedTuple
from gameProgram import GameProgram, GameCommand

from game.database.gameDatabase import GameDatabase, CommandInfo, ResourceInfo, BuildingInfo
from eventManager import EventManager

class CommandState:
    def __init__(self, info : CommandInfo):
        self.info: CommandInfo = info
        self.unlocked: bool = False

class BuildingState:
    def __init__(self, info : BuildingInfo):
        self.info: BuildingInfo = info
        self.totalCount: int = 0
        self.activeCount: int = 0
        self.unlocked: bool = False
        
class ResourceState:
    def __init__(self, info : ResourceInfo):
        self.info: ResourceInfo = info
        self.income: float = 0
        self.storage: float = 0
        self.count: float = 0
        self.unlocked: bool = False

class EventState:
    def __init__(self, info : EventInfo):
        self.info: EventInfo = info
        self.completed: bool = False
        self.displayed: bool = False
        self.triggered: bool = False
        self.ongoing: bool = False
        
class ResourceList:
    def __init__(self, r : Dict[str, float]):
        self.r = r

class GameState:
    def __init__(self, database : GameDatabase):
        self.database: GameDatabase = database
        
        self.commands: Dict[str, CommandState] = {}
        for cInfp in database.commands.values():
            self.commands[cInfp.name] = CommandState(cInfp)
            
        self.buildings: Dict[str, BuildingState] = {}
        for bInfo in database.buildings.values():
            bState = BuildingState(bInfo)
            bState.totalCount = database.params.startingBuildings[bInfo.name]
            bState.activeCount = bState.totalCount
            self.buildings[bInfo.name] = bState

        self.resources: Dict[str, ResourceState] = {}
        for rInfo in database.resources.values():
            rState = ResourceState(rInfo)
            rState.count = database.params.startingResources[rInfo.name]
            self.resources[rInfo.name] = rState

        self.events: Dict[str, EventState] = {}
        for eInfo in database.events.values():
            eState = EventState(eInfo)
            self.events[eInfo.name] = eState

        self.programs: List[GameProgram] = []
        for i in range(0, database.params.maxProgramCount):
            self.programs.append(GameProgram(self))
        self.programs[0].assignedProcessors = 1
        self.freeProcessorCount = 0
        
        for objectToUnlock in self.database.params.startingUnlocks:
            self.unlock(objectToUnlock)

        loadDebugProgram = False
        if loadDebugProgram:
            self.programs[0].commands.append(GameCommand(self.commands["Sell Cloud Compute"].info))
            self.programs[0].commands.append(GameCommand(self.commands["Gather Regolith"].info))
            self.programs[0].commands.append(GameCommand(self.commands["Idle"].info))
            self.programs[0].commands.append(GameCommand(self.commands["Sell Cloud Compute"].info))
            self.programs[0].commands[2].maxCount = 5
        
        self.eventManager: EventManager = EventManager(self)
        self.activeEvents: List[EventState] = []
        self.ongoingEvents: List[EventState] = []
        self.ticks: int = 0

        self.step()

    def unlock(self, objectToUnlock : str):
        if objectToUnlock in self.commands:
            self.commands[objectToUnlock].unlocked = True
        elif objectToUnlock in self.buildings:
            self.buildings[objectToUnlock].unlocked = True
        elif objectToUnlock in self.resources:
            self.resources[objectToUnlock].unlocked = True
        else:
            print('objectToUnlock not found: ' + objectToUnlock)
            
    def updateStorage(self):
        for r in self.resources.values():
            r.storage = self.database.params.startingStorage[r.info.name]
        
        # update storage from buildings
        for bState in self.buildings.values():
            bInfo = bState.info
            
            # buildings with storage should not have upkeep requirements
            for rName, stor in bInfo.storage.items():
                rState = self.resources[rName]
                rState.storage += bState.activeCount * stor

    def updateIncomeAndBuildingProduction(self):
        # because buildings that fail upkeep don't produce resources, we must handle
        # income and building proudction in the same function.
    
        for r in self.resources.values():
            r.income = 0
        
        # update resources from buildings
        for bState in self.buildings.values():
            if bState.activeCount == 0:
                continue
            
            bInfo = bState.info
            bName = bInfo.name
            
            bProduction = self.getBuildingProduction(bName)
            bUpkeep = self.getBuildingUpkeep(bName)
            if len(bUpkeep.r) == 0:
                for rName, prod in bInfo.production.items():
                    rState = self.resources[rName]
                    rState.income += prod * bState.activeCount
                    rState.count += prod * bState.activeCount
            else:
                # upkeep counts as negative income
                for rName, cost in bUpkeep.r.items():
                    rState = self.resources[rName]
                    rState.income -= cost * bState.activeCount
                    
                # for buildings with upkeep, production is handled one at a time in case
                # there are not sufficient upkeep resources.
                for i in range(0, bState.activeCount):
                    if self.canAffordCost(bUpkeep):
                        self.spendResources(bUpkeep)
                    else:
                        continue

                    for rName, prod in bInfo.production.items():
                        rState = self.resources[rName]
                        rState.income += prod
                        rState.count += prod
        
    def updateProcessorAllocation(self):
        self.freeProcessorCount = self.resources['Processors'].storage
        for program in self.programs:
            if program.assignedProcessors <= self.freeProcessorCount:
                self.freeProcessorCount -= program.assignedProcessors
            else:
                program.assignedProcessors = self.freeProcessorCount
                self.freeProcessorCount = 0
        
    def updateStorageAndProcessors(self):
        self.updateStorage()
        self.updateProcessorAllocation()
        
    def runAllPrograms(self):
        for program in self.programs:
            if program.assignedProcessors > 0:
                program.step()

    def step(self):
        self.updateStorageAndProcessors()
        self.updateIncomeAndBuildingProduction()
        self.runAllPrograms()
        
        # cap all resources to their storage capacity
        for rState in self.resources.values():
            rState.count = min(rState.count, rState.storage)
            
        self.eventManager.step()
        
        self.ticks += 1
        
    def getBuildingProduction(self, buildingName : str) -> ResourceList:
        b = self.buildings[buildingName]
        
        prod: Dict[str, float] = {}
        for rName, production in b.info.production.items():
            prod[rName] = production
            
        return ResourceList(prod)

    def getBuildingUpkeep(self, buildingName : str) -> ResourceList:
        b = self.buildings[buildingName]
        
        upkeep: Dict[str, float] = {}
        costMultiplier = 1.0
        for rName, upkeepCost in b.info.upkeep.items():
            upkeep[rName] = upkeepCost * costMultiplier
            
        return ResourceList(upkeep)
    
    def getBuildingCost(self, buildingName : str) -> ResourceList:
        b = self.buildings[buildingName]
        
        costs: Dict[str, float] = {}
        costMultiplier = pow(b.info.costScaling, b.totalCount)
        for rName, baseCost in b.info.baseCost.items():
            costs[rName] = math.floor(baseCost * costMultiplier)
            
        return ResourceList(costs)
    
    def getCommandCost(self, commandName : str) -> ResourceList:
        c = self.commands[commandName]
        
        costs: Dict[str, float] = {}
        costMultiplier = 1.0
        for rName, baseCost in c.info.cost.items():
            costs[rName] = baseCost * costMultiplier
            
        return ResourceList(costs)
    
    def canAffordCost(self, cost : ResourceList) -> bool:
        for r, v in cost.r.items():
            if v > self.resources[r].count:
                return False
        return True

    def spendResources(self, resourceList : ResourceList):
        for r, v in resourceList.r.items():
            if v > self.resources[r].count:
                print('cannot afford cost')
                return
            self.resources[r].count -= v

    def attemptPurchaseBuilding(self, buildingName):
        buildingCost = self.getBuildingCost(buildingName)
        if not self.canAffordCost(buildingCost):
            print('cannot afford ' + buildingName)
            return
        
        self.spendResources(buildingCost)
        self.buildings[buildingName].totalCount += 1
        self.buildings[buildingName].activeCount += 1
        self.updateStorageAndProcessors()
        
    def runCommand(self, commandName):
        cState : CommandState = self.commands[commandName]
        cInfo = cState.info
        
        commandCost = self.getCommandCost(commandName)
        if not self.canAffordCost(commandCost):
            return
        self.spendResources(commandCost)
            
        for rName, v in cInfo.production.items():
            r = self.resources[rName]
            r.count = min(r.count + v, r.storage)

if __name__ == "__main__":
    print('testing game state')
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    for i in range(0, 10):
        state.step()