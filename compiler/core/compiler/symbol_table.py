from typing import Dict, Any, Optional
from ..ir.types import ValueType
from ..errors import TypeError

class SymbolTable:
    """
    Manages symbol resolution and scoping for variables and functions.
    Supports hierarchical scopes.
    """
    def __init__(self, parent: Optional['SymbolTable'] = None):
        self.parent = parent
        self.symbols: Dict[str, dict] = {}
        
        # Pre-populate built-ins in the global scope
        if self.parent is None:
            self._register_builtins()
            
    def _register_builtins(self):
        # Default shader inputs
        self.define("x", ValueType.SCALAR)
        self.define("y", ValueType.SCALAR)
        self.define("z", ValueType.SCALAR)
        
    def define(self, name: str, value_type: ValueType, metadata: Any = None):
        if name in self.symbols:
            raise TypeError(f"Symbol '{name}' is already defined in the current scope.")
        self.symbols[name] = {"type": value_type, "metadata": metadata}
        
    def lookup(self, name: str) -> dict:
        if name in self.symbols:
            return self.symbols[name]
        if self.parent:
            return self.parent.lookup(name)
        raise TypeError(f"Undefined symbol: '{name}'")
        
    def resolve_type(self, name: str) -> ValueType:
        return self.lookup(name)["type"]
