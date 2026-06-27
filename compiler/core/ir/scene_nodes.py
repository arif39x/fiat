import uuid
from dataclasses import dataclass, field
from .math_expr import Expr, Vec3
from typing import Optional

@dataclass(frozen=True, kw_only=True)
class Transform:
    """
    Represents a local coordinate transform.
    Future improvements will include matrix composition.
    """
    position: tuple[float, float, float] = (0.0, 0.0, 0.0)
    rotation: tuple[float, float, float] = (0.0, 0.0, 0.0)
    scale: tuple[float, float, float] = (1.0, 1.0, 1.0)

@dataclass(frozen=True, kw_only=True)
class MaterialNode:
    """Material properties for a scene node."""
    id: int = 1  # 0 usually background, 1 default
    color: tuple[float, float, float] = (1.0, 1.0, 1.0)
    roughness: float = 0.5
    emissive: float = 0.0

@dataclass(frozen=True, kw_only=True)
class SceneNode:
    """Base class for all scene graph nodes."""
    id: str = field(default_factory=lambda: str(uuid.uuid4()), compare=False, hash=False)
    transform: Transform = field(default_factory=Transform)
    material: Optional[MaterialNode] = field(default=None)

@dataclass(frozen=True, kw_only=True)
class SphereNode(SceneNode):
    radius: Expr

@dataclass(frozen=True, kw_only=True)
class BoxNode(SceneNode):
    size: Vec3

@dataclass(frozen=True, kw_only=True)
class UnionNode(SceneNode):
    left: SceneNode
    right: SceneNode
