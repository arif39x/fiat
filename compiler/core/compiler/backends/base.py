from abc import ABC, abstractmethod
from ...ir.math_expr import Expr
from ...ir.scene_nodes import SceneNode

class Backend(ABC):
    """Abstract base class for all compilation backends."""
    
    @abstractmethod
    def compile_expr(self, expr: Expr):
        """Compiles a Math IR expression into the backend's AST or intermediate representation."""
        pass
        
    @abstractmethod
    def compile_scene(self, node: SceneNode):
        """Compiles a SceneGraph node into the backend's AST or intermediate representation."""
        pass
        
    @abstractmethod
    def emit_source(self, ast_node) -> str:
        """Emits final source code string from the backend's AST."""
        pass
