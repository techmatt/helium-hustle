
from __future__ import annotations
from ast import Str

import json
import os
import math
from typing import Dict, List, NamedTuple, Set
from game.core.gameProgram import GameProgram, GameCommand

from game.database.gameDatabase import (ResourceList, GameDatabase, GameParams, CommandInfo,
    ResourceInfo, BuildingInfo, EventInfo, ResearchInfo, ProjectInfo, AdversaryInfo, IdeologyInfo,
    DefenderInfo)

from game.core.eventManager import EventManager
from game.core.modifierManager import ModifierManager

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
        self.income: float = 0 # will be negative if more of the resource is consumed than produced
        self.storage: float = 0
        self.count: float = 0
        self.unlocked: bool = False

class EventState:
    def __init__(self, info : EventInfo):
        self.info: EventInfo = info
        
        self.triggered: bool = False # events conditions have been met
        self.completed: bool = False # a triggered event has been resolved
        self.ongoing: bool = False # an event with income or lasting effects is ongoing
        
        self.displayed: bool = False # displayed is only used by the UI and is not relevant for the game state
        self.timestampStr : str = None

class ResearchState:
    def __init__(self, info : ResearchInfo):
        self.info: ResearchInfo = info
        self.purchased: bool = False
        self.unlocked: bool = False

class ProjectState:
    def __init__(self, info : ProjectInfo):
        self.info: ProjectInfo = info
        self.purchaseCount: int = 0
        self.resourcePayments: Dict[str, float] = {}
        self.progress: float = 0.0

class IdeologyState:
    def __init__(self, info : IdeologyInfo):
        self.info: IdeologyInfo = info
        self.totalScore: float = 0
        self.rank: int = 0
        self.localRankScore: float = 0
        self.localRankThreshold: float = 0
        
class AdversaryState:
    def __init__(self, info : AdversaryInfo):
        self.info: AdversaryInfo = info
        self.unlocked: bool = False
        self.strength: float = 0
        self.spawnRate: float = 0
        self.ticksToSurge: float = 0
        self.nextSurgeStrength: float = 0
        self.decayRate: float = 0
        self.effectiveness: float = 0
        
class DefenderState:
    def __init__(self, info : DefenderInfo):
        self.info: DefenderInfo = info
        self.rState: ResourceState = None
        self.unlocked: bool = False
        self.decayRate: float = 0
    
class DirtyState:
    def __init__(self):
        self.events: bool = True
        self.projects: bool = True
        
class GameState:
    def __init__(self, database : GameDatabase):
        self.database: GameDatabase = database
        self.params: GameParams = self.database.params # quick access to params
        
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

        self.research: Dict[str, ResearchState] = {}
        self.purchasedResearch : Set[str] = set()
        for rInfo in database.research.values():
            rState = ResearchState(rInfo)
            self.research[rInfo.name] = rState

        self.projects: Dict[str, ProjectState] = {}
        for pInfo in database.projects.values():
            pState = ProjectState(pInfo)
            for r in pInfo.resourceRates.keys():
                pState.resourcePayments[r] = 0
            self.projects[pInfo.name] = pState
            
        self.events: Dict[str, EventState] = {}
        for eInfo in database.events.values():
            eState = EventState(eInfo)
            self.events[eInfo.name] = eState

        self.ideologies: Dict[str, IdeologyState] = {}
        for iInfo in database.ideologies.values():
            iState = IdeologyState(iInfo)
            self.ideologies[iInfo.name] = iState
            
        self.adversaries: Dict[str, AdversaryState] = {}
        for aInfo in database.adversaries.values():
            aState = AdversaryState(aInfo)
            self.adversaries[aInfo.name] = aState

        self.defenders: Dict[str, DefenderState] = {}
        self.defendersByCategory: Dict[str, DefenderState] = {}
        for dInfo in database.defenders.values():
            dState = DefenderState(dInfo)
            dState.rState = self.resources[dInfo.name]
            self.defenders[dInfo.name] = dState
            self.defendersByCategory[dInfo.category] = dState

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
        
        adversaryDebug = True
        if adversaryDebug:
            for dState in self.defenders.values():
                dState.unlocked = True
            for aState in self.adversaries.values():
                aState.unlocked = True
                
        self.debugSkipEvents = True

        self.eventManager: EventManager = EventManager(self)
        self.activeEvents: List[EventState] = []
        self.ongoingEvents: List[EventState] = []
        self.ticks: int = 0
        self.ticksUntilProcessorCycle: int = 0
        self.ticksUntilArmyCycle: int = 0
        self.dirty: DirtyState = DirtyState()

        self.step()

    def convertPerTickToPerSecond(self, tickRate : float) -> float:
        return tickRate * self.database.params.ticksPerPlayerSecond
    
    def convertTicksToYears(self, ticks : float) -> float:
        return ticks / self.database.params.ticksPerGameYear
    
    def unlock(self, objectToUnlock : str):
        if objectToUnlock in self.commands:
            self.commands[objectToUnlock].unlocked = True
        elif objectToUnlock in self.buildings:
            self.buildings[objectToUnlock].unlocked = True
        elif objectToUnlock in self.resources:
            self.resources[objectToUnlock].unlocked = True
        else:
            print('objectToUnlock not found: ' + objectToUnlock)
            
    def updateProjectPayments(self):
        for pState in self.projects.values():
            pInfo = pState.info
            for rName, rPayment in pState.resourcePayments.items():
                r = self.resources[rName]
                r.income -= rPayment
                totalPayment = min(rPayment, r.count)
                r.count -= totalPayment
                pState.progress += totalPayment * pInfo.resourceRates[rName]
            
            totalProjectCost = self.getProjectCost(pInfo.name)
            if pState.progress >= totalProjectCost:
                self.completeProject(pInfo.name)

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
        
        # update resources from ongoing events
        for e in self.ongoingEvents:
            for rName, prod in e.info.income.items():
                rState = self.resources[rName]
                rState.income += prod
                rState.count += prod

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

    def armySurge(self, adversaryName : str):
        aState = self.adversaries[adversaryName]
        aState.strength += aState.nextSurgeStrength
        aState.ticksToSurge = aState.info.surgeIntervalTicks
        aState.nextSurgeStrength *= aState.info.surgeScaleFactor
        aState.spawnRate *= aState.info.surgeScaleFactor
        
    def updateArmies(self):
        # process adversary growth, attrition, and surges
        for aState in self.adversaries.values():
            if not aState.unlocked:
                continue
            
            # adversaries decay over time
            aState.strength *= (1.0 - aState.decayRate)
            
            # adversaries spawn new troops
            aState.strength += aState.spawnRate

            # periodically, armies surge, creating a burst of new troops
            aState.ticksToSurge -= self.params.ticksPerArmyCycle
            if aState.ticksToSurge < 0:
                self.armySurge(aState.info.name)
                
        # process defender attrition
        for dState in self.defenders.values():
            if not dState.unlocked:
                continue
            
            # defenders decay over time
            dState.rState.count *= dState.decayRate
        
        # process all army fights
        for aState in self.adversaries.values():
            dState = self.defendersByCategory[aState.info.category]
            
            totalDefenders = dState.rState.count
            totalAttackers = aState.strength
                    
            totalArmies = totalDefenders + totalAttackers
            activeFighters = totalArmies * self.params.armyFightRatio
            activeFighters = min(activeFighters, totalAttackers)
            activeFighters = min(activeFighters, totalDefenders)
            
            dState.rState.count -= activeFighters
            aState.strength -= activeFighters

        for aState in self.adversaries.values():
            totalDefenders = dState.rState.count
            totalAttackers = aState.strength
            totalArmies = totalDefenders + totalAttackers
            if totalArmies == 0:
                aState.effectiveness = 0
            else:
                aState.effectiveness = totalAttackers / totalArmies
            
    def step(self):
        self.updateStorageAndProcessors()
        self.updateIncomeAndBuildingProduction()
        
        if self.ticksUntilProcessorCycle > 0:
            self.ticksUntilProcessorCycle -= 1
        else:
            self.runAllPrograms()
            self.ticksUntilProcessorCycle = self.database.params.ticksPerProcessorCycle
        
        self.updateProjectPayments()

        if self.ticksUntilArmyCycle > 0:
            self.ticksUntilArmyCycle -= 1
        else:
            self.updateArmies()
            self.ticksUntilArmyCycle = self.database.params.ticksPerProcessorCycle
        
        # cap all resources to their storage capacity
        for rState in self.resources.values():
            rState.count = min(rState.count, rState.storage)
            
        for iState in self.ideologies.values():
            ModifierManager.updateIdeologyRank(self, iState)
            
        self.eventManager.step()
        
        self.ticks += 1
    
    def completeProject(self, projectName : str):
        pState = self.projects[projectName]
        pState.purchaseCount += 1
        pState.progress = 0
        self.dirty.projects = True
    
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
    
    def getProjectCost(self, projectName : str) -> float:
        return ModifierManager.getProjectCost(self, projectName)
        
    def getBuildingCost(self, buildingName : str) -> ResourceList:
        return ModifierManager.getBuildingCost(self, buildingName)
    
    def getResearchCost(self, researchName : str) -> ResourceList:
        r = self.research[researchName]
        
        costs: Dict[str, float] = {}
        costMultiplier = 1.0
        for resourceName, resourceCost in r.info.cost.items():
            costs[resourceName] = math.floor(resourceCost * costMultiplier)
            
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
        
    def attemptPurchaseResearch(self, researchName):
        researchCost = self.getResearchCost(researchName)
        if not self.canAffordCost(researchCost):
            print('cannot afford ' + researchName)
            return
        
        self.spendResources(researchCost)
        self.research[researchName].purchased = True
        self.purchasedResearch.add(researchName)
        self.updateStorageAndProcessors()
        
    def checkResearch(self, rName : str) -> bool:
        if not rName in self.research:
            print('research not found', rName)
        return rName in self.purchasedResearch
        
    def runCommand(self, commandName):
        cState : CommandState = self.commands[commandName]
        cInfo = cState.info
        
        commandCost = self.getCommandCost(commandName)
        if not self.canAffordCost(commandCost):
            return
        self.spendResources(commandCost)
            
        for rName, v in cInfo.production.items():
            r = self.resources[rName]
            r.count += v
            
        for iName, v in cInfo.ideology.items():
            i = self.ideologies[iName]
            i.totalScore += v
            
    def processEventOption(self, eState : EventState, option : Str):
        eInfo = eState.info
        
        # ongoing events don't have options.
        # completed events can't be processed a second time.
        if eState in self.ongoingEvents or eState.completed:
            return
        
        if not(eState in self.activeEvents):
            print('event not active', eState.info.name)
            return
        
        if option != 'OK' and option not in eInfo.options:
            print('invalid option', option)
            return
        
        if option == 'Maybe later':
            return
        
        if option == 'OK' or option == 'Maybe later':
            pass
        elif option == 'Spend all boredom':
            self.resources['Boredom'].count = 0
        else:
            print('option not found', option)
            
        self.activeEvents.remove(eState)
        eState.completed = True
        self.dirty.events = True
        

if __name__ == "__main__":
    print('testing game state')
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    database = GameDatabase('gameDatabase.json')
    state = GameState(database)
    for i in range(0, 10):
        state.step()