from typing import Any
from ..ir.scene_nodes import SceneNode, SphereNode, BoxNode, UnionNode

class SceneVisitor:
    """Base visitor for SceneGraph nodes."""
    
    def visit(self, node: SceneNode, *args, **kwargs) -> Any:
        method_name = f"visit_{type(node).__name__}"
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(node, *args, **kwargs)

    def generic_visit(self, node: SceneNode, *args, **kwargs) -> Any:
        raise NotImplementedError(f"No visit_{type(node).__name__} method defined in {type(self).__name__}")
