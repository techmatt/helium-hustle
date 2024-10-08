
from __future__ import annotations
from ast import Str

import json
import os
import math
from typing import Dict, List, NamedTuple
from game.core.gameProgram import GameProgram, GameCommand

from game.database.gameDatabase import (ResourceList, GameDatabase, CommandInfo,
    ResourceInfo, BuildingInfo, EventInfo, ResearchInfo, ProjectInfo)
from game.core.eventManager import EventManager

class ModifierManager:
    @staticmethod
    def getBuildingCost(state : GameState, buildingName : str) -> ResourceList:
        b = state.buildings[buildingName]
        
        costs: Dict[str, float] = {}
        costMultiplier = pow(b.info.costScaling, b.totalCount)
        
        for resourceName, baseCost in b.info.baseCost.items():
            rScaleFactor = 1.0
            if resourceName == 'Regolith' and state.checkResearch('Efficient Building Design'):
                rScaleFactor *= 0.9

            costs[resourceName] = math.floor(baseCost * costMultiplier * rScaleFactor)
            
        return ResourceList(costs)
    
    def getProjectCost(state : GameState, projectName : str) -> float:
        pState = state.projects[projectName]
        info = pState.info
        
        costMultiplier = pow(info.costScaling, pState.purchaseCount)
        cost = costMultiplier * info.baseCost
        
        return math.floor(cost)
    
    def updateIdeologyRank(state : GameState, iState : IdeologyState):
        params = state.database.params
        
        s = abs(iState.totalScore)
        iState.rank = 0
        while True:
            iState.localRankThreshold = params.baseIdeologyCost * math.pow(params.ideologyScaleFactor, iState.rank)
            if s >= iState.localRankThreshold:
                iState.rank += 1
                s -= iState.localRankThreshold
            else:
                break
        iState.localRankScore = s
        if iState.totalScore < 0.0:
            iState.rank = -iState.rank
