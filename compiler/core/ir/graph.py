from dataclasses import dataclass
from .scene_nodes import SceneNode

@dataclass
class Universe:
    """Top-level container for all scene objects in the programmable universe."""
    nodes: list[SceneNode]
