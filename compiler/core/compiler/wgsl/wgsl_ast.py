from dataclasses import dataclass
from typing import List, Optional

@dataclass(frozen=True)
class WGSLNode:
    pass

@dataclass(frozen=True)
class WGSLConstant(WGSLNode):
    value: float
    type_name: str = "f32"
    
    def emit(self) -> str:
        # e.g., f32(10.0)
        v = float(self.value)
        return f"{self.type_name}({v})" if "." in str(v) else f"{self.type_name}({v}.0)"

@dataclass(frozen=True)
class WGSLVariable(WGSLNode):
    name: str
    
    def emit(self) -> str:
        return self.name

@dataclass(frozen=True)
class WGSLBinaryExpr(WGSLNode):
    op: str
    left: WGSLNode
    right: WGSLNode
    
    def emit(self) -> str:
        return f"({self.left.emit()} {self.op} {self.right.emit()})"

@dataclass(frozen=True)
class WGSLCallExpr(WGSLNode):
    func_name: str
    args: List[WGSLNode]
    
    def emit(self) -> str:
        args_str = ", ".join(arg.emit() for arg in self.args)
        return f"{self.func_name}({args_str})"

@dataclass(frozen=True)
class WGSLVec3(WGSLNode):
    x: WGSLNode
    y: WGSLNode
    z: WGSLNode
    
    def emit(self) -> str:
        return f"vec3<f32>({self.x.emit()}, {self.y.emit()}, {self.z.emit()})"

@dataclass(frozen=True)
class WGSLFunction(WGSLNode):
    name: str
    params: List[tuple[str, str]] # (name, type)
    return_type: str
    body: str # For now body can be string, ideally a list of WGSLStatements
    
    def emit(self) -> str:
        params_str = ", ".join(f"{name}: {t}" for name, t in self.params)
        return f"fn {self.name}({params_str}) -> {self.return_type} {{\n    {self.body}\n}}"

@dataclass(frozen=True)
class SdfResult(WGSLNode):
    distance: WGSLNode
    material_id: WGSLNode
    
    def emit(self) -> str:
        return f"vec2<f32>({self.distance.emit()}, {self.material_id.emit()})"
