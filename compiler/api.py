from firewall import validate_equation
from core.analysis.complexity import analyze_complexity
from core.compiler.optimizer.canonicalize import canonicalize
from core.compiler.optimizer.passes import optimize
from core.compiler.pass_manager import PassManager
from core.compiler.type_checker import infer_type
from core.compiler.wgsl.backend import WGSLBackend
from core.parser.lower_to_ir import parse_string_to_ir
from core.runtime.validation import validate_expr
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from templates.wgsl_module import build_wgsl_module

app = FastAPI()


class EquationRequest(BaseModel):
    equation: str


pm = PassManager()
pm.add_pass("Type Inference", infer_type)
pm.add_pass("Semantic Validation", validate_expr)
pm.add_pass("Canonicalization", canonicalize)
pm.add_pass("Optimization", optimize)
pm.add_pass("Complexity Analysis", analyze_complexity)

backend = WGSLBackend()


@app.post("/compile_sdf")
async def compile_sdf(req: EquationRequest):
    try:
        norm_eq = validate_equation(req.equation)

        ir_expr = parse_string_to_ir(norm_eq)

        final_ir = pm.run(ir_expr)

        wgsl_ast = backend.compile_expr(final_ir)
        wgsl_expr_string = wgsl_ast.emit()

        score = getattr(final_ir, "complexity_score", 50)
        max_steps = max(64, 256 - int(score * 0.5))
        full_code = build_wgsl_module(wgsl_expr_string, max_steps)

        return {"status": "success", "wgsl": full_code}

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Compilation error: {str(e)}")
