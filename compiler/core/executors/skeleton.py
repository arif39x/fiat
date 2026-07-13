from __future__ import annotations

from ..animation.math import Quaternion, Transform
from ..animation.skeleton import Joint, Skeleton


def execute_skeleton(params: dict) -> Skeleton:
    joints_data = params.get("joints", [])
    joints = []
    for jd in joints_data:
        t = jd.get("translation", [0.0, 0.0, 0.0])
        joints.append(Joint(
            name=jd["name"],
            parent_index=jd["parent"],
            local_transform=Transform(
                translation=(t[0], t[1], t[2]),
                rotation=Quaternion.identity(),
                scale=(1.0, 1.0, 1.0),
            ),
        ))
    return Skeleton(joints=joints, name="llm_generated")
