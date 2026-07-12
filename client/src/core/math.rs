use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quaternion {
    pub fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        let cx = (x * 0.5).cos();
        let sx = (x * 0.5).sin();
        let cy = (y * 0.5).cos();
        let sy = (y * 0.5).sin();
        let cz = (z * 0.5).cos();
        let sz = (z * 0.5).sin();
        Self {
            w: cx * cy * cz + sx * sy * sz,
            x: sx * cy * cz - cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
        }
    }

    #[allow(dead_code)]
    pub fn from_axis_angle(axis: (f32, f32, f32), angle: f32) -> Self {
        let (ax, ay, az) = axis;
        let length = (ax * ax + ay * ay + az * az).sqrt();
        if length < 1e-8 {
            return Self::identity();
        }
        let s = (angle * 0.5).sin() / length;
        Self {
            w: (angle * 0.5).cos(),
            x: ax * s,
            y: ay * s,
            z: az * s,
        }
    }

    pub fn normalize(self) -> Self {
        let n = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if n < 1e-8 {
            return Self::identity();
        }
        Self {
            w: self.w / n,
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
        }
    }

    #[allow(dead_code)]
    pub fn conjugate(self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    #[allow(dead_code)]
    pub fn inverse(self) -> Self {
        self.conjugate().normalize()
    }

    #[allow(dead_code)]
    pub fn rotate_vector(self, v: (f32, f32, f32)) -> (f32, f32, f32) {
        let qv = Quaternion {
            w: 0.0,
            x: v.0,
            y: v.1,
            z: v.2,
        };
        let result = self * qv * self.conjugate();
        (result.x, result.y, result.z)
    }

    pub fn to_matrix(self) -> [f32; 16] {
        let xx = self.x * self.x * 2.0;
        let yy = self.y * self.y * 2.0;
        let zz = self.z * self.z * 2.0;
        let xy = self.x * self.y * 2.0;
        let xz = self.x * self.z * 2.0;
        let yz = self.y * self.z * 2.0;
        let wx = self.w * self.x * 2.0;
        let wy = self.w * self.y * 2.0;
        let wz = self.w * self.z * 2.0;
        [
            1.0 - (yy + zz),
            xy + wz,
            xz - wy,
            0.0,
            xy - wz,
            1.0 - (xx + zz),
            yz + wx,
            0.0,
            xz + wy,
            yz - wx,
            1.0 - (xx + yy),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ]
    }

    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z;
        let mut other = other;
        if dot < 0.0 {
            dot = -dot;
            other = Self {
                w: -other.w,
                x: -other.x,
                y: -other.y,
                z: -other.z,
            };
        }
        if dot > 0.9995 {
            let result = Self {
                w: self.w + t * (other.w - self.w),
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
            };
            return result.normalize();
        }
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        let s1 = sin_theta / sin_theta_0;
        let s0 = theta.cos() - dot * s1;
        Self {
            w: s0 * self.w + s1 * other.w,
            x: s0 * self.x + s1 * other.x,
            y: s0 * self.y + s1 * other.y,
            z: s0 * self.z + s1 * other.z,
        }
        .normalize()
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: (f32, f32, f32),
    pub rotation: Quaternion,
    #[allow(dead_code)]
    pub scale: (f32, f32, f32),
}

impl Transform {
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            translation: (0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: (1.0, 1.0, 1.0),
        }
    }

    pub fn to_matrix(self) -> [f32; 16] {
        let mut rot = self.rotation.to_matrix();
        rot[3] = self.translation.0;
        rot[7] = self.translation.1;
        rot[11] = self.translation.2;
        rot
    }
}

pub fn mat4_to_quaternion(m: &[f32; 16]) -> Quaternion {
    let m00 = m[0]; let m01 = m[4]; let m02 = m[8];
    let m10 = m[1]; let m11 = m[5]; let m12 = m[9];
    let m20 = m[2]; let m21 = m[6]; let m22 = m[10];
    let trace = m00 + m11 + m22;
    if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        Quaternion {
            w: 0.25 / s,
            x: (m21 - m12) * s,
            y: (m02 - m20) * s,
            z: (m10 - m01) * s,
        }
    } else if m00 > m11 && m00 > m22 {
        let s = 2.0 * (1.0 + m00 - m11 - m22).sqrt();
        Quaternion {
            w: (m21 - m12) / s,
            x: 0.25 * s,
            y: (m01 + m10) / s,
            z: (m02 + m20) / s,
        }
    } else if m11 > m22 {
        let s = 2.0 * (1.0 + m11 - m00 - m22).sqrt();
        Quaternion {
            w: (m02 - m20) / s,
            x: (m01 + m10) / s,
            y: 0.25 * s,
            z: (m12 + m21) / s,
        }
    } else {
        let s = 2.0 * (1.0 + m22 - m00 - m11).sqrt();
        Quaternion {
            w: (m10 - m01) / s,
            x: (m02 + m20) / s,
            y: (m12 + m21) / s,
            z: 0.25 * s,
        }
    }
    .normalize()
}

pub fn multiply_mat4(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0f32; 16];
    for i in 0..4 {
        for j in 0..4 {
            result[i * 4 + j] = a[i * 4 + 0] * b[0 * 4 + j]
                + a[i * 4 + 1] * b[1 * 4 + j]
                + a[i * 4 + 2] * b[2 * 4 + j]
                + a[i * 4 + 3] * b[3 * 4 + j];
        }
    }
    result
}

pub fn forward_kinematics(
    local_transforms: &[Transform],
    parent_indices: &[i32],
) -> Vec<Transform> {
    let n = local_transforms.len();
    let mut global = vec![Transform::identity(); n];
    for i in 0..n {
        if parent_indices[i] < 0 {
            global[i] = local_transforms[i];
        } else {
            let parent = &global[parent_indices[i] as usize];
            let combined = multiply_mat4(&parent.to_matrix(), &local_transforms[i].to_matrix());
            global[i] = Transform {
                translation: (combined[3], combined[7], combined[11]),
                rotation: mat4_to_quaternion(&combined),
                scale: (1.0, 1.0, 1.0),
            };
        }
    }
    global
}
