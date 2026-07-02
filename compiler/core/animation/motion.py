from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Tuple

from .math import Quaternion
from .skeleton import Pose, Skeleton


@dataclass
class MotionClip:
    skeleton: Skeleton
    frames: List[List[Quaternion]] = field(default_factory=list)
    root_positions: List[Tuple[float, float, float]] = field(default_factory=list)
    fps: float = 30.0
    loop: bool = False

    def frame_count(self) -> int:
        return len(self.frames)

    def duration(self) -> float:
        if self.frame_count() == 0:
            return 0.0
        return self.frame_count() / self.fps

    def get_pose(self, frame_index: int) -> Pose:
        idx = max(0, min(frame_index, self.frame_count() - 1))
        return Pose(
            skeleton=self.skeleton,
            joint_rotations=self.frames[idx],
            root_translation=self.root_positions[idx] if idx < len(self.root_positions) else (0.0, 0.0, 0.0),
        )

    def sample(self, time: float) -> Pose:
        if self.frame_count() == 0:
            return Pose(
                skeleton=self.skeleton,
                joint_rotations=[Quaternion.identity()] * self.skeleton.joint_count(),
            )
        if self.loop:
            time = time % self.duration()
        t = time / self.duration() if self.duration() > 0 else 0.0
        idx = int(t * (self.frame_count() - 1))
        frac = t * (self.frame_count() - 1) - idx
        idx = max(0, min(idx, self.frame_count() - 2))
        a = self.frames[idx]
        b = self.frames[idx + 1]
        rotations = []
        for qa, qb in zip(a, b):
            rotations.append(qa.slerp(qb, frac))
        root_a = self.root_positions[idx] if idx < len(self.root_positions) else (0.0, 0.0, 0.0)
        root_b = self.root_positions[idx + 1] if idx + 1 < len(self.root_positions) else (0.0, 0.0, 0.0)
        root_translation = (
            root_a[0] + (root_b[0] - root_a[0]) * frac,
            root_a[1] + (root_b[1] - root_a[1]) * frac,
            root_a[2] + (root_b[2] - root_a[2]) * frac,
        )
        return Pose(
            skeleton=self.skeleton,
            joint_rotations=rotations,
            root_translation=root_translation,
        )

    def validate(self) -> List[str]:
        errors = []
        skel_errors = self.skeleton.validate()
        errors.extend(skel_errors)
        if self.frame_count() == 0:
            errors.append("Motion clip has zero frames")
            return errors
        jc = self.skeleton.joint_count()
        for i, frame in enumerate(self.frames):
            if len(frame) != jc:
                errors.append(f"Frame {i} has {len(frame)} joint rotations, expected {jc}")
            for q in frame:
                n = math.sqrt(q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z)
                if abs(n - 1.0) > 0.01:
                    errors.append(f"Frame {i} has non-unit quaternion (norm={n})")
        if self.root_positions and len(self.root_positions) != self.frame_count():
            errors.append(
                f"root_positions count {len(self.root_positions)} != frame count {self.frame_count()}"
            )
        return errors

    def to_dict(self) -> dict:
        return {
            "skeleton": self.skeleton.to_dict(),
            "fps": self.fps,
            "loop": self.loop,
            "frames": [
                [{"w": q.w, "x": q.x, "y": q.y, "z": q.z} for q in frame]
                for frame in self.frames
            ],
            "root_positions": [list(rp) for rp in self.root_positions],
        }

    @staticmethod
    def from_dict(data: dict) -> MotionClip:
        import math as _math
        skeleton = Skeleton.from_dict(data["skeleton"])
        frames = [
            [Quaternion(q["w"], q["x"], q["y"], q["z"]) for q in frame]
            for frame in data["frames"]
        ]
        root_positions = [tuple(rp) for rp in data.get("root_positions", [])]
        return MotionClip(
            skeleton=skeleton,
            frames=frames,
            root_positions=root_positions,
            fps=data.get("fps", 30.0),
            loop=data.get("loop", False),
        )
