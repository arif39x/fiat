from ...ir.math_expr import Add, Constant, Expr, Mul, Variable
from ...visitor.expr_visitor import ExprTransformer


def _expr_weight(expr: Expr) -> tuple:
    """
    Returns a sorting weight for an expression.
    Constants come first, then Variables alphabetically, then more complex trees.
    """
    if isinstance(expr, Constant):
        return (0, expr.value)
    if isinstance(expr, Variable):
        return (1, expr.name)
    return (2, type(expr).__name__)


class CanonicalizePass(ExprTransformer):
    def visit_Add(self, expr: Add) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        # Sort commutative operands
        if _expr_weight(left) > _expr_weight(right):
            left, right = right, left

        return Add(left=left, right=right, span=expr.span, value_type=expr.value_type)

    def visit_Mul(self, expr: Mul) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        if _expr_weight(left) > _expr_weight(right):
            left, right = right, left

        return Mul(left=left, right=right, span=expr.span, value_type=expr.value_type)


def canonicalize(expr: Expr) -> Expr:
    visitor = CanonicalizePass()
    return visitor.visit(expr)
