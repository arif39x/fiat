from __future__ import annotations

from typing import Any, Dict, Optional

from ..animation.math import Quaternion
from ..animation.motion import MotionClip
from ..animation.skeleton import Skeleton


def generate_motion(
    prompt: str,
    target_skeleton: Optional[Skeleton] = None,
    seed: Optional[int] = None,
) -> MotionClip:
    jc = target_skeleton.joint_count() if target_skeleton else 1
    frames = [[Quaternion.identity()] * jc for _ in range(30)]
    root_positions = [(0.0, 0.0, 0.0)] * 30

    return MotionClip(
        skeleton=target_skeleton or Skeleton(name="fallback"),
        frames=frames,
        root_positions=root_positions,
        fps=30.0,
        loop=True,
    )
