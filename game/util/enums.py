
from enum import Enum, auto

class GameWindowMode(Enum):
    COMMANDS = auto()
    BUILDINGS = auto()
    RESEARCH = auto()
    ACHIEVEMENTS = auto()
    OPTIONS = auto()
    
class Ideology(Enum):
    MILITARY = auto()
    SCIENCE = auto()
    CULTURE = auto()