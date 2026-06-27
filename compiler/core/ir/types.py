from enum import Enum, auto
from typing import ForwardRef, Union


class ValueType(Enum):
    SCALAR = auto()
    VEC2 = auto()
    VEC3 = auto()
    BOOL = auto()


Expr = ForwardRef("Expr")
SceneNode = ForwardRef("SceneNode")

# Type alias for any IR node
IRNode = Union[Expr, SceneNode]
