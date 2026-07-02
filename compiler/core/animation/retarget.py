from __future__ import annotations

from typing import Dict, List, Optional, Tuple

from .math import Quaternion, Transform
from .motion import MotionClip
from .skeleton import Pose, Skeleton


def build_bone_name_map(
    source_skeleton: Skeleton,
    target_skeleton: Skeleton,
    known_maps: Optional[Dict[str, Dict[str, str]]] = None,
    source_name: str = "",
) -> Dict[int, int]:
    mapping: Dict[int, int] = {}
    known = known_maps or {}
    key = source_name or source_skeleton.name
    if key in known:
        name_map = known[key]
        for src_name, tgt_name in name_map.items():
            si = source_skeleton.index_of(src_name)
            ti = target_skeleton.index_of(tgt_name)
            if si >= 0 and ti >= 0:
                mapping[si] = ti

    if not mapping:
        mapping = _auto_match(source_skeleton, target_skeleton)

    return mapping


def _auto_match(
    source_skeleton: Skeleton,
    target_skeleton: Skeleton,
) -> Dict[int, int]:
    mapping: Dict[int, int] = {}
    src_names_lower = [n.lower() for n in source_skeleton.joint_names()]
    tgt_names_lower = [n.lower() for n in target_skeleton.joint_names()]

    for si, s_name in enumerate(src_names_lower):
        best_score = 0.0
        best_ti = -1
        for ti, t_name in enumerate(tgt_names_lower):
            if ti in mapping.values():
                continue
            score = _name_similarity(s_name, t_name)
            if score > best_score:
                best_score = score
                best_ti = ti
        if best_score > 0.3:
            mapping[si] = best_ti

    src_root = source_skeleton.root_index()
    tgt_root = target_skeleton.root_index()
    if src_root not in mapping:
        mapping[src_root] = tgt_root

    return mapping


def _name_similarity(a: str, b: str) -> float:
    if a == b:
        return 1.0
    if a in b or b in a:
        return 0.8
    a_parts = set(a.replace("_", " ").replace(".", " ").split())
    b_parts = set(b.replace("_", " ").replace(".", " ").split())
    if not a_parts or not b_parts:
        return 0.0
    intersection = a_parts & b_parts
    return len(intersection) / max(len(a_parts), len(b_parts))


def retarget_clip(
    clip: MotionClip,
    target_skeleton: Skeleton,
    bone_map: Dict[int, int],
) -> MotionClip:
    tgt_joint_count = target_skeleton.joint_count()
    new_frames = []
    new_root_positions = []

    for frame_idx, src_rotations in enumerate(clip.frames):
        new_rotations = [Quaternion.identity()] * tgt_joint_count
        for src_idx, tgt_idx in bone_map.items():
            if tgt_idx < tgt_joint_count and src_idx < len(src_rotations):
                src_joint = clip.skeleton.joints[src_idx]
                tgt_joint = target_skeleton.joints[tgt_idx]

                src_rest = src_joint.local_transform.rotation
                tgt_rest = tgt_joint.local_transform.rotation

                src_local = src_rotations[src_idx]
                tgt_local = tgt_rest.inverse() * (src_rest * src_local * src_rest.inverse()) * tgt_rest
                new_rotations[tgt_idx] = tgt_local.normalize()

        new_frames.append(new_rotations)

        if clip.root_positions:
            src_root = clip.root_positions[frame_idx] if frame_idx < len(clip.root_positions) else (0.0, 0.0, 0.0)
            scale = _limb_scale_factor(clip.skeleton, target_skeleton, bone_map)
            new_root_positions.append(
                (src_root[0] * scale, src_root[1] * scale, src_root[2] * scale)
            )

    return MotionClip(
        skeleton=target_skeleton,
        frames=new_frames,
        root_positions=new_root_positions if new_root_positions else clip.root_positions,
        fps=clip.fps,
        loop=clip.loop,
    )


def _limb_scale_factor(
    src_skeleton: Skeleton,
    tgt_skeleton: Skeleton,
    bone_map: Dict[int, int],
) -> float:
    total_ratio = 0.0
    count = 0
    for src_idx, tgt_idx in bone_map.items():
        src_len = _bone_length(src_skeleton, src_idx)
        tgt_len = _bone_length(tgt_skeleton, tgt_idx)
        if src_len > 0.01:
            total_ratio += tgt_len / src_len
            count += 1
    if count == 0:
        return 1.0
    return total_ratio / count


def _bone_length(skeleton: Skeleton, joint_index: int) -> float:
    joint = skeleton.joints[joint_index]
    t = joint.local_transform.translation
    return (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]) ** 0.5
