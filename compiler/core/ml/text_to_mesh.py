from __future__ import annotations

import math
from typing import Any, Dict, Optional, Tuple


def generate_mesh(
    prompt: str,
    seed: Optional[int] = None,
) -> Tuple[dict, dict]:
    vertices = [
        {"position": [-0.5, -0.5, -0.5], "normal": [0.0, 0.0, -1.0], "uv": [0.0, 0.0],
         "bone_weights": [1.0, 0.0, 0.0, 0.0], "bone_indices": [0, 0, 0, 0]},
        {"position": [0.5, -0.5, -0.5], "normal": [0.0, 0.0, -1.0], "uv": [1.0, 0.0],
         "bone_weights": [1.0, 0.0, 0.0, 0.0], "bone_indices": [0, 0, 0, 0]},
        {"position": [0.5, 0.5, -0.5], "normal": [0.0, 0.0, -1.0], "uv": [1.0, 1.0],
         "bone_weights": [1.0, 0.0, 0.0, 0.0], "bone_indices": [0, 0, 0, 0]},
        {"position": [-0.5, 0.5, -0.5], "normal": [0.0, 0.0, -1.0], "uv": [0.0, 1.0],
         "bone_weights": [1.0, 0.0, 0.0, 0.0], "bone_indices": [0, 0, 0, 0]},
    ]
    indices = [0, 1, 2, 0, 2, 3]
    skeleton = {
        "name": "fallback",
        "joints": [
            {"name": "root", "parent_index": -1,
             "local_transform": {"translation": [0.0, 0.0, 0.0],
                                  "rotation": {"w": 1.0, "x": 0.0, "y": 0.0, "z": 0.0},
                                  "scale": [1.0, 1.0, 1.0]},
             "joint_limits": None},
        ],
    }
    return {"vertices": vertices, "indices": indices}, skeleton
