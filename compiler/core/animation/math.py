from __future__ import annotations

import math
from dataclasses import dataclass
from typing import List, Tuple


@dataclass(frozen=True)
class Quaternion:
    w: float
    x: float
    y: float
    z: float

    @staticmethod
    def identity() -> Quaternion:
        return Quaternion(1.0, 0.0, 0.0, 0.0)

    @staticmethod
    def from_axis_angle(axis: Tuple[float, float, float], angle: float) -> Quaternion:
        ax, ay, az = axis
        length = math.sqrt(ax * ax + ay * ay + az * az)
        if length < 1e-8:
            return Quaternion.identity()
        s = math.sin(angle * 0.5) / length
        return Quaternion(math.cos(angle * 0.5), ax * s, ay * s, az * s)

    def normalize(self) -> Quaternion:
        n = math.sqrt(self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z)
        if n < 1e-8:
            return Quaternion.identity()
        return Quaternion(self.w / n, self.x / n, self.y / n, self.z / n)

    def conjugate(self) -> Quaternion:
        return Quaternion(self.w, -self.x, -self.y, -self.z)

    def inverse(self) -> Quaternion:
        return self.conjugate().normalize()

    def __mul__(self, other: Quaternion) -> Quaternion:
        return Quaternion(
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        )

    def rotate_vector(self, v: Tuple[float, float, float]) -> Tuple[float, float, float]:
        qv = Quaternion(0.0, v[0], v[1], v[2])
        result = self * qv * self.conjugate()
        return (result.x, result.y, result.z)

    def to_matrix(self) -> List[float]:
        xx = self.x * self.x * 2.0
        yy = self.y * self.y * 2.0
        zz = self.z * self.z * 2.0
        xy = self.x * self.y * 2.0
        xz = self.x * self.z * 2.0
        yz = self.y * self.z * 2.0
        wx = self.w * self.x * 2.0
        wy = self.w * self.y * 2.0
        wz = self.w * self.z * 2.0
        return [
            1.0 - (yy + zz), xy + wz, xz - wy, 0.0,
            xy - wz, 1.0 - (xx + zz), yz + wx, 0.0,
            xz + wy, yz - wx, 1.0 - (xx + yy), 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]

    def slerp(self, other: Quaternion, t: float) -> Quaternion:
        dot = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
        flip = dot < 0.0
        if flip:
            dot = -dot
            other = Quaternion(-other.w, -other.x, -other.y, -other.z)
        if dot > 0.9995:
            result = Quaternion(
                self.w + t * (other.w - self.w),
                self.x + t * (other.x - self.x),
                self.y + t * (other.y - self.y),
                self.z + t * (other.z - self.z),
            ).normalize()
            return result
        theta_0 = math.acos(dot)
        theta = theta_0 * t
        sin_theta = math.sin(theta)
        sin_theta_0 = math.sin(theta_0)
        s1 = sin_theta / sin_theta_0
        s0 = math.cos(theta) - dot * s1
        return Quaternion(
            s0 * self.w + s1 * other.w,
            s0 * self.x + s1 * other.x,
            s0 * self.y + s1 * other.y,
            s0 * self.z + s1 * other.z,
        ).normalize()


@dataclass
class Transform:
    translation: Tuple[float, float, float]
    rotation: Quaternion
    scale: Tuple[float, float, float]

    @staticmethod
    def identity() -> Transform:
        return Transform((0.0, 0.0, 0.0), Quaternion.identity(), (1.0, 1.0, 1.0))

    def to_matrix(self) -> List[float]:
        rot = self.rotation.to_matrix()
        tx, ty, tz = self.translation
        rot[3] = tx
        rot[7] = ty
        rot[11] = tz
        return rot


def mat4_to_quaternion(m: List[float]) -> Quaternion:
    m00, m01, m02 = m[0], m[4], m[8]
    m10, m11, m12 = m[1], m[5], m[9]
    m20, m21, m22 = m[2], m[6], m[10]
    trace = m00 + m11 + m22
    if trace > 0.0:
        s = 0.5 / math.sqrt(trace + 1.0)
        return Quaternion(0.25 / s, (m21 - m12) * s, (m02 - m20) * s, (m10 - m01) * s).normalize()
    elif m00 > m11 and m00 > m22:
        s = 2.0 * math.sqrt(1.0 + m00 - m11 - m22)
        return Quaternion((m21 - m12) / s, 0.25 * s, (m01 + m10) / s, (m02 + m20) / s).normalize()
    elif m11 > m22:
        s = 2.0 * math.sqrt(1.0 + m11 - m00 - m22)
        return Quaternion((m02 - m20) / s, (m01 + m10) / s, 0.25 * s, (m12 + m21) / s).normalize()
    else:
        s = 2.0 * math.sqrt(1.0 + m22 - m00 - m11)
        return Quaternion((m10 - m01) / s, (m02 + m20) / s, (m12 + m21) / s, 0.25 * s).normalize()


def multiply_mat4(a: List[float], b: List[float]) -> List[float]:
    result = [0.0] * 16
    for i in range(4):
        for j in range(4):
            result[i * 4 + j] = (
                a[i * 4 + 0] * b[0 * 4 + j]
                + a[i * 4 + 1] * b[1 * 4 + j]
                + a[i * 4 + 2] * b[2 * 4 + j]
                + a[i * 4 + 3] * b[3 * 4 + j]
            )
    return result


def forward_kinematics(
    local_transforms: List[Transform],
    parent_indices: List[int],
) -> List[Transform]:
    n = len(local_transforms)
    global_transforms = [Transform.identity() for _ in range(n)]
    for i in range(n):
        local_mat = local_transforms[i].to_matrix()
        if parent_indices[i] < 0:
            global_transforms[i] = local_transforms[i]
        else:
            parent_mat = global_transforms[parent_indices[i]].to_matrix()
            combined = multiply_mat4(parent_mat, local_mat)
            global_transforms[i] = Transform(
                (combined[3], combined[7], combined[11]),
                mat4_to_quaternion(combined),
                (1.0, 1.0, 1.0),
            )
    return global_transforms
