import pytest
from core.ir.math_expr import Constant, Variable, Add, Mul, Div, Pow, Sqrt
from core.parser.lower_to_ir import parse_string_to_ir
from core.runtime.validation import validate_expr
from core.compiler.optimizer.passes import optimize
from core.compiler.wgsl.backend import emit_expr

def test_parsing():
    ir = parse_string_to_ir("x + y * 2")
    assert isinstance(ir, Add)
    assert isinstance(ir.right, Mul)
    assert ir.right.right == Constant(value=2.0)

def test_validation_invalid_var():
    ir = parse_string_to_ir("x + a")
    with pytest.raises(Exception, match="Invalid variable name"):
        validate_expr(ir)

def test_validation_pow_limit():
    ir = parse_string_to_ir("x ** 10")
    with pytest.raises(Exception, match="exceeds maximum"):
        validate_expr(ir)

def test_validation_div_zero():
    ir = parse_string_to_ir("x / 0")
    with pytest.raises(Exception):
        validate_expr(ir)

def test_optimization_constant_folding():
    ir = parse_string_to_ir("2 + 2")
    opt = optimize(ir)
    assert opt == Constant(value=4.0)

def test_optimization_algebraic():
    ir = parse_string_to_ir("x * 1 + 0")
    opt = optimize(ir)
    assert opt == Variable(name="x")

def test_wgsl_emission():
    ir = parse_string_to_ir("sqrt(x*x + y*y) - 10")
    wgsl = emit_expr(ir)
    # Note: the exact nested parens might depend on the IR structure
    assert "sqrt" in wgsl
    assert "f32(10.0)" in wgsl

def test_complex_pipeline():
    expr_str = "sqrt(x*x + y*y + z*z) - 10 + (2 * 0)"
    ir = parse_string_to_ir(expr_str)
    validate_expr(ir)
    opt = optimize(ir)
    wgsl = emit_expr(opt)
    
    assert "(2 * 0)" not in wgsl
    assert "sqrt" in wgsl
    assert "f32(10.0)" in wgsl
