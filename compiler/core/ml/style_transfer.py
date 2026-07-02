from __future__ import annotations

from typing import Optional

from ..animation.math import Quaternion
from ..animation.motion import MotionClip
from ..animation.skeleton import Skeleton


def apply_style_transfer(
    source_clip: MotionClip,
    style_prompt: str,
    style_reference: Optional[MotionClip] = None,
) -> MotionClip:
    return source_clip
