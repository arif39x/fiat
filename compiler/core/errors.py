from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class SourceSpan:
    line: int
    column: int

    def __str__(self) -> str:
        return f"Line {self.line}, Column {self.column}"


class CompilerError(Exception):

    def __init__(
        self,
        message: str,
        span: Optional[SourceSpan] = None,
        node_type: Optional[str] = None,
    ):
        self.message = message
        self.span = span
        self.node_type = node_type
        super().__init__(self._format_message())

    def _format_message(self) -> str:
        parts = []
        if self.span:
            parts.append(f"{self.span}:")
        if self.node_type:
            parts.append(f"[{self.node_type}]")
        parts.append(self.message)
        return " ".join(parts)


class ParseError(CompilerError):
    pass


class ValidationError(CompilerError):
    pass


class TypeError(CompilerError):
    pass


class BackendError(CompilerError):
    pass


class ComplexityError(CompilerError):
    pass
