from __future__ import annotations

import math
from typing import List, Optional, Tuple

from .math import Quaternion
from .motion import MotionClip
from .skeleton import Pose, Skeleton


def validate_pose(pose: Pose) -> List[str]:
    errors: List[str] = []
    errors.extend(pose.skeleton.validate())
    for i, rot in enumerate(pose.joint_rotations):
        n = math.sqrt(rot.w * rot.w + rot.x * rot.x + rot.y * rot.y + rot.z * rot.z)
        if abs(n - 1.0) > 0.01:
            errors.append(f"Joint {i} has non-unit quaternion (norm={n})")
    return errors


def validate_clip(clip: MotionClip) -> List[str]:
    errors: List[str] = []
    errors.extend(clip.skeleton.validate())
    if clip.frame_count() == 0:
        errors.append("Motion clip has zero frames")
        return errors
    jc = clip.skeleton.joint_count()
    for i, frame in enumerate(clip.frames):
        if len(frame) != jc:
            errors.append(f"Frame {i} has {len(frame)} joint rotations, expected {jc}")
        for qi, q in enumerate(frame):
            n = math.sqrt(q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z)
            if abs(n - 1.0) > 0.01:
                errors.append(f"Frame {i} joint {qi} non-unit quaternion (norm={n})")
    if clip.root_positions and len(clip.root_positions) != clip.frame_count():
        errors.append(
            f"root_positions count {len(clip.root_positions)} != frame count {clip.frame_count()}"
        )
    return errors


def validate_joint_limits(pose: Pose) -> List[str]:
    errors: List[str] = []
    for i, rot in enumerate(pose.joint_rotations):
        limits = pose.skeleton.joints[i].joint_limits
        if limits is None:
            continue
        euler = quaternion_to_euler(rot)
        for axis_idx, (name, min_val, max_val) in enumerate(
            [("x", limits.get("min_x", -math.pi), limits.get("max_x", math.pi)),
             ("y", limits.get("min_y", -math.pi), limits.get("max_y", math.pi)),
             ("z", limits.get("min_z", -math.pi), limits.get("max_z", math.pi))]
        ):
            if euler[axis_idx] < min_val or euler[axis_idx] > max_val:
                errors.append(
                    f"Joint {i} '{pose.skeleton.joints[i].name}' {name} angle {euler[axis_idx]:.3f} "
                    f"outside limits [{min_val:.3f}, {max_val:.3f}]"
                )
    return errors


def quaternion_to_euler(q: Quaternion) -> Tuple[float, float, float]:
    sinr_cosp = 2.0 * (q.w * q.x + q.y * q.z)
    cosr_cosp = 1.0 - 2.0 * (q.x * q.x + q.y * q.y)
    roll = math.atan2(sinr_cosp, cosr_cosp)
    sinp = 2.0 * (q.w * q.y - q.z * q.x)
    pitch = math.asin(sinp) if abs(sinp) <= 1.0 else (math.pi / 2 if sinp > 0 else -math.pi / 2)
    siny_cosp = 2.0 * (q.w * q.z + q.x * q.y)
    cosy_cosp = 1.0 - 2.0 * (q.y * q.y + q.z * q.z)
    yaw = math.atan2(siny_cosp, cosy_cosp)
    return (roll, pitch, yaw)
