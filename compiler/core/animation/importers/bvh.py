from __future__ import annotations

import math
import re
from typing import List, Optional, TextIO, Tuple

from ..math import Quaternion
from ..motion import MotionClip
from ..skeleton import Joint, Pose, Skeleton, Transform


def parse_bvh(file: TextIO) -> MotionClip:
    lines = file.readlines()
    tokenizer = _tokenize(lines)
    tokens = list(tokenizer)
    pos = 0

    def peek() -> Optional[Tuple[str, str]]:
        if pos < len(tokens):
            return tokens[pos]
        return None

    def consume(expected: Optional[str] = None) -> Tuple[str, str]:
        nonlocal pos
        if pos >= len(tokens):
            raise ValueError("Unexpected end of BVH file")
        tok = tokens[pos]
        pos += 1
        if expected is not None and tok[1] != expected:
            raise ValueError(f"Expected '{expected}', got '{tok[1]}'")
        return tok

    _type, _val = consume()
    if _val != "HIERARCHY":
        raise ValueError("Expected HIERARCHY section")

    joints: List[Joint] = []
    joint_order: List[str] = []
    channels_per_joint: List[List[str]] = []
    stack: List[int] = []

    _type, _val = consume()
    if _val != "ROOT":
        raise ValueError("Expected ROOT")

    def parse_joint(parent_index: int) -> int:
        _type, name = consume()
        joint_order.append(name)
        idx = len(joints)
        joints.append(
            Joint(
                name=name,
                parent_index=parent_index,
                local_transform=Transform.identity(),
            )
        )
        _type, _val = consume()
        if _val != "{":
            raise ValueError("Expected '{' after joint name")
        offset = [0.0, 0.0, 0.0]
        channels: List[str] = []
        while True:
            _type, val = consume()
            if val == "}":
                break
            elif val == "OFFSET":
                _, x = consume();
                offset[0] = float(x)
                _, y = consume();
                offset[1] = float(y)
                _, z = consume();
                offset[2] = float(z)
            elif val == "CHANNELS":
                _, count_str = consume()
                count = int(count_str)
                for _ in range(count):
                    _, ch = consume()
                    channels.append(ch)
            elif val == "JOINT" or val == "End":
                if val == "End":
                    _, _ = consume()  # site name
                child_idx = parse_joint(idx)
            else:
                if val != "{":
                    pass
        joints[idx] = Joint(
            name=joint_order[idx],
            parent_index=parent_index,
            local_transform=Transform(
                translation=(offset[0], offset[1], offset[2]),
                rotation=Quaternion.identity(),
                scale=(1.0, 1.0, 1.0),
            ),
        )
        channels_per_joint.append(channels)
        return idx

    parse_joint(-1)

    _type, _val = consume()
    if _val != "MOTION":
        raise ValueError("Expected MOTION section")

    _, _ = consume()  # Frames:
    _, num_frames_str = consume()
    num_frames = int(num_frames_str)

    _, _ = consume()  # Frame Time:
    _, frame_time_str = consume()
    fps = 1.0 / float(frame_time_str)

    frames: List[List[Quaternion]] = []
    root_positions: List[Tuple[float, float, float]] = []

    for _ in range(num_frames):
        rotations = [Quaternion.identity()] * len(joints)
        root_pos = (0.0, 0.0, 0.0)
        for i, channels in enumerate(channels_per_joint):
            values = []
            for _ in range(len(channels)):
                _, v = consume()
                values.append(float(v))
            ch_idx = 0
            pos_offset = [0.0, 0.0, 0.0]
            rot_euler = [0.0, 0.0, 0.0]
            for ch in channels:
                if ch == "Xposition":
                    pos_offset[0] = values[ch_idx]
                elif ch == "Yposition":
                    pos_offset[1] = values[ch_idx]
                elif ch == "Zposition":
                    pos_offset[2] = values[ch_idx]
                elif ch == "Xrotation":
                    rot_euler[0] = math.radians(values[ch_idx])
                elif ch == "Yrotation":
                    rot_euler[1] = math.radians(values[ch_idx])
                elif ch == "Zrotation":
                    rot_euler[2] = math.radians(values[ch_idx])
                ch_idx += 1
            if i == 0:
                root_pos = (pos_offset[0], pos_offset[1], pos_offset[2])
            q = euler_to_quaternion(rot_euler[0], rot_euler[1], rot_euler[2])
            rotations[i] = q
        frames.append(rotations)
        root_positions.append(root_pos)

    skeleton = Skeleton(joints=joints, name="imported")
    return MotionClip(
        skeleton=skeleton,
        frames=frames,
        root_positions=root_positions,
        fps=fps,
        loop=False,
    )


def _tokenize(lines: List[str]):
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        for token in re.findall(r'[A-Za-z_][A-Za-z0-9_.]*|[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?|[{}:]', line):
            yield ("token", token)


def euler_to_quaternion(roll: float, pitch: float, yaw: float) -> Quaternion:
    cr = math.cos(roll * 0.5)
    sr = math.sin(roll * 0.5)
    cp = math.cos(pitch * 0.5)
    sp = math.sin(pitch * 0.5)
    cy = math.cos(yaw * 0.5)
    sy = math.sin(yaw * 0.5)
    return Quaternion(
        w=cr * cp * cy + sr * sp * sy,
        x=sr * cp * cy - cr * sp * sy,
        y=cr * sp * cy + sr * cp * sy,
        z=cr * cp * sy - sr * sp * cy,
    )
