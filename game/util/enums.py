
from enum import Enum, auto

class GameWindowMode(Enum):
    COMMANDS = auto()
    BUILDINGS = auto()
    RESEARCH = auto()
    PROJECTS = auto()
    IDEOLOGIES = auto()
    STATS = auto()
    ACHIEVEMENTS = auto()
    OPTIONS = auto()

class Ideology(Enum):
    MILITARY = auto()
    SCIENCE = auto()
    CULTURE = auto()