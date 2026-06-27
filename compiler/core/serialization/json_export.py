import json
from dataclasses import is_dataclass, asdict
from ..ir.graph import Universe
from ..ir.math_expr import Expr
from ..ir.scene_nodes import SceneNode, Transform, MaterialNode
from ..ir.types import ValueType
from ..errors import SourceSpan

def ir_to_dict(obj):
    """
    Recursively converts IR nodes into JSON-serializable dictionaries.
    """
    if isinstance(obj, ValueType):
        return obj.name
    
    if isinstance(obj, SourceSpan):
        return {"line": obj.line, "column": obj.column}

    if isinstance(obj, (Expr, SceneNode)):
        result = {"type": type(obj).__name__}
        for field_name, value in obj.__dict__.items():
            if field_name == "id": # Keep UUIDs
                result[field_name] = value
                continue
            if value is not None:
                result[field_name] = ir_to_dict(value)
        return result
    
    if isinstance(obj, (Transform, MaterialNode)):
        return asdict(obj)
    
    if isinstance(obj, list):
        return [ir_to_dict(item) for item in obj]
    
    if isinstance(obj, tuple):
        return [ir_to_dict(item) for item in obj]
    
    return obj

def export_universe(universe: Universe) -> str:
    """
    Serializes the Universe graph to a JSON string.
    """
    data = {
        "nodes": [ir_to_dict(node) for node in universe.nodes]
    }
    return json.dumps(data, indent=2)
