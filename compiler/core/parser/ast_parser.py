import ast


def parse(code: str) -> ast.AST:
    return ast.parse(code, mode="eval")
