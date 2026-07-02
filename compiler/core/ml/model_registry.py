from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, Optional


@dataclass
class ModelEntry:
    name: str
    version: str
    endpoint: str
    fallback: str
    timeout_seconds: float = 30.0
    enabled: bool = True


class ModelRegistry:
    def __init__(self):
        self._models: Dict[str, ModelEntry] = {}

    def register(self, key: str, entry: ModelEntry):
        self._models[key] = entry

    def get(self, key: str) -> Optional[ModelEntry]:
        return self._models.get(key)

    def is_enabled(self, key: str) -> bool:
        entry = self.get(key)
        return entry is not None and entry.enabled

    def resolve(self, key: str) -> tuple[str, str]:
        entry = self.get(key)
        if entry is None or not entry.enabled:
            return ("fallback", "")
        return ("model", entry.endpoint)


_default_registry: Optional[ModelRegistry] = None


def get_default_registry() -> ModelRegistry:
    global _default_registry
    if _default_registry is None:
        _default_registry = ModelRegistry()
        _default_registry.register(
            "text_to_motion",
            ModelEntry(
                name="motion-diffusion",
                version="0.1.0",
                endpoint="http://127.0.0.1:8082/v1/generate_motion",
                fallback="procedural_idle",
            ),
        )
        _default_registry.register(
            "text_to_mesh",
            ModelEntry(
                name="shap-e",
                version="0.1.0",
                endpoint="http://127.0.0.1:8082/v1/generate_mesh",
                fallback="template_mesh",
            ),
        )
        _default_registry.register(
            "pose_interpolation",
            ModelEntry(
                name="pose-transformer",
                version="0.1.0",
                endpoint="http://127.0.0.1:8082/v1/interpolate_pose",
                fallback="procedural_slerp",
                timeout_seconds=10.0,
            ),
        )
        _default_registry.register(
            "style_transfer",
            ModelEntry(
                name="adain-motion",
                version="0.1.0",
                endpoint="http://127.0.0.1:8082/v1/style_transfer",
                fallback="identity",
            ),
        )
    return _default_registry
