from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple

from .math import Quaternion, Transform


@dataclass
class Joint:
    name: str
    parent_index: int
    local_transform: Transform
    joint_limits: Optional[Dict[str, Tuple[float, float]]] = None


@dataclass
class Skeleton:
    joints: List[Joint] = field(default_factory=list)
    name: str = ""

    def joint_count(self) -> int:
        return len(self.joints)

    def joint_names(self) -> List[str]:
        return [j.name for j in self.joints]

    def index_of(self, name: str) -> int:
        for i, j in enumerate(self.joints):
            if j.name == name:
                return i
            return -1

    def parent_indices(self) -> List[int]:
        return [j.parent_index for j in self.joints]

    def root_index(self) -> int:
        for i, j in enumerate(self.joints):
            if j.parent_index < 0:
                return i
            return 0

    def validate(self) -> List[str]:
        errors = []
        for i, j in enumerate(self.joints):
            if j.parent_index >= i:
                errors.append(f"Joint '{j.name}' parent_index {j.parent_index} >= own index {i}")
            if j.parent_index < -1:
                errors.append(f"Joint '{j.name}' has invalid parent_index {j.parent_index}")
            if not j.name:
                errors.append(f"Joint at index {i} has empty name")
        roots = sum(1 for j in self.joints if j.parent_index < 0)
        if roots == 0:
            errors.append("Skeleton has no root joint")
        if roots > 1:
            errors.append(f"Skeleton has {roots} root joints")
        return errors

    def to_dict(self) -> dict:
        return {
            "name": self.name,
            "joints": [
                {
                    "name": j.name,
                    "parent_index": j.parent_index,
                    "local_transform": {
                        "translation": list(j.local_transform.translation),
                        "rotation": {
                            "w": j.local_transform.rotation.w,
                            "x": j.local_transform.rotation.x,
                            "y": j.local_transform.rotation.y,
                            "z": j.local_transform.rotation.z,
                        },
                        "scale": list(j.local_transform.scale),
                    },
                    "joint_limits": j.joint_limits,
                }
                for j in self.joints
            ],
        }

    @staticmethod
    def from_dict(data: dict) -> Skeleton:
        joints = []
        for jd in data["joints"]:
            rot = jd["local_transform"]["rotation"]
            joints.append(
                Joint(
                    name=jd["name"],
                    parent_index=jd["parent_index"],
                    local_transform=Transform(
                        translation=tuple(jd["local_transform"]["translation"]),
                        rotation=Quaternion(rot["w"], rot["x"], rot["y"], rot["z"]),
                        scale=tuple(jd["local_transform"]["scale"]),
                    ),
                    joint_limits=jd.get("joint_limits"),
                )
            )
        return Skeleton(joints=joints, name=data.get("name", ""))


@dataclass
class Pose:
    skeleton: Skeleton
    joint_rotations: List[Quaternion]
    root_translation: Tuple[float, float, float] = (0.0, 0.0, 0.0)

    def __post_init__(self):
        if len(self.joint_rotations) != self.skeleton.joint_count():
            raise ValueError(
                f"joint_rotations count {len(self.joint_rotations)} "
                f"does not match skeleton joint count {self.skeleton.joint_count()}"
            )

    def local_transforms(self) -> List[Transform]:
        result = []
        for i, rot in enumerate(self.joint_rotations):
            base = self.skeleton.joints[i].local_transform
            result.append(
                Transform(
                    translation=base.translation if i > 0 else self.root_translation,
                    rotation=rot,
                    scale=base.scale,
                )
            )
        return result
