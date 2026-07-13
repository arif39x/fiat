from __future__ import annotations

import math
import random
from typing import Any, Dict, List, Tuple


def execute_texture(params: dict) -> dict:
    w = params.get("width", 512)
    h = params.get("height", 512)
    colors = params.get("colors", [{"position": 0.0, "rgb": [0.5, 0.5, 0.5]}])
    pattern = params.get("pattern", "solid")
    pp = params.get("pattern_params", {})

    pixels = []
    for y in range(h):
        for x in range(w):
            u, v = x / w, y / h
            color = _sample_pattern(u, v, pattern, pp, colors)
            pixels.extend([int(c * 255) for c in color] + [255])

    return {
        "width": w,
        "height": h,
        "channels": 4,
        "data": pixels,
    }


def _lerp_color(colors: list, t: float) -> tuple:
    t = max(0.0, min(1.0, t))
    for i in range(len(colors) - 1):
        if colors[i]["position"] <= t <= colors[i + 1]["position"]:
            local = (t - colors[i]["position"]) / (colors[i + 1]["position"] - colors[i]["position"] + 1e-8)
            return (
                colors[i]["rgb"][0] + (colors[i + 1]["rgb"][0] - colors[i]["rgb"][0]) * local,
                colors[i]["rgb"][1] + (colors[i + 1]["rgb"][1] - colors[i]["rgb"][1]) * local,
                colors[i]["rgb"][2] + (colors[i + 1]["rgb"][2] - colors[i]["rgb"][2]) * local,
            )
    return tuple(colors[-1]["rgb"])


def _sample_pattern(u: float, v: float, pattern: str, pp: dict, colors: list) -> tuple:
    freq = pp.get("frequency", 10)
    dist = pp.get("distortion", 0)

    if pattern == "solid":
        return _lerp_color(colors, 0.0)

    elif pattern == "gradient":
        return _lerp_color(colors, u)

    elif pattern == "wood_grain":
        val = math.sin(u * freq + math.sin(v * freq + dist) * 0.5) * 0.5 + 0.5
        return _lerp_color(colors, val)

    elif pattern == "noise":
        val = math.sin(u * freq * 2) * math.cos(v * freq * 2) * 0.5 + 0.5
        return _lerp_color(colors, val)

    elif pattern == "checker":
        val = (math.floor(u * freq) + math.floor(v * freq)) % 2
        return _lerp_color(colors, val)

    elif pattern == "stripe":
        angle = pp.get("angle", 0.0)
        pu = u * math.cos(angle) - v * math.sin(angle)
        val = (pu * freq) % 1.0
        return _lerp_color(colors, val)

    else:
        return _lerp_color(colors, 0.5)
