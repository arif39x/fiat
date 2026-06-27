import time
from typing import List, Callable, Any
from ..ir.math_expr import Expr
from ..ir.scene_nodes import SceneNode
from ..errors import CompilerError

class PassManager:
    """Manages the execution pipeline of compiler passes."""
    def __init__(self):
        self.passes: List[tuple[str, Callable[[Any], Any]]] = []
        self.diagnostics = {}

    def add_pass(self, name: str, pass_func: Callable[[Any], Any]):
        """Registers a pass in the pipeline."""
        self.passes.append((name, pass_func))

    def run(self, node: Any) -> Any:
        """Executes all passes sequentially and collects timing diagnostics."""
        current_node = node
        
        for name, pass_func in self.passes:
            start_time = time.time()
            
            try:
                # Some passes return a new node, some modify in place/just validate
                result = pass_func(current_node)
                if result is not None:
                    current_node = result
            except CompilerError as e:
                # Add pass name to the error context if needed, then re-raise
                # Optionally log failure
                raise e
                
            elapsed = time.time() - start_time
            self.diagnostics[name] = elapsed
            
        return current_node

    def print_diagnostics(self):
        """Prints timing info for all passes."""
        print("--- Compiler Pass Diagnostics ---")
        for name, elapsed in self.diagnostics.items():
            print(f"{name:.<30} {elapsed * 1000:.2f} ms")
        print("---------------------------------")
