use crate::core::math::Quaternion;
use crate::core::skeleton::{Pose, Skeleton};

#[allow(dead_code)]
pub struct FABRIKSolver {
    pub tolerance: f32,
    pub max_iterations: u32,
}

impl FABRIKSolver {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tolerance: 0.01,
            max_iterations: 20,
        }
    }

    #[allow(dead_code)]
    pub fn solve(
        &self,
        skeleton: &Skeleton,
        pose: &Pose,
        target: (f32, f32, f32),
        chain: &[usize],
    ) -> Pose {
        if chain.len() < 2 {
            return pose.clone();
        }

        let mut positions: Vec<(f32, f32, f32)> = chain.iter().map(|&idx| {
            let base = &skeleton.joints[idx].local_transform;
            let parent_pos = if skeleton.joints[idx].parent_index >= 0 {
                let pidx = skeleton.joints[idx].parent_index as usize;
                let p_local = &pose.joint_rotations[pidx];
                let p_base = &skeleton.joints[pidx].local_transform;
                let p_rot = p_local.to_matrix();
                let p_trans = if pidx == 0 {
                    pose.root_translation
                } else {
                    (p_base.translation[0], p_base.translation[1], p_base.translation[2])
                };
                let px = p_rot[3] + p_trans.0;
                let py = p_rot[7] + p_trans.1;
                let pz = p_rot[11] + p_trans.2;
                (px, py, pz)
            } else {
                pose.root_translation
            };
            let rel = (base.translation[0], base.translation[1], base.translation[2]);
            (parent_pos.0 + rel.0, parent_pos.1 + rel.1, parent_pos.2 + rel.2)
        }).collect();

        let lengths: Vec<f32> = (1..positions.len()).map(|i| {
            let dx = positions[i].0 - positions[i-1].0;
            let dy = positions[i].1 - positions[i-1].1;
            let dz = positions[i].2 - positions[i-1].2;
            (dx*dx + dy*dy + dz*dz).sqrt()
        }).collect();

        let total_length: f32 = lengths.iter().sum();
        let dx = target.0 - positions[0].0;
        let dy = target.1 - positions[0].1;
        let dz = target.2 - positions[0].2;
        let dist_to_target = (dx*dx + dy*dy + dz*dz).sqrt();
        if dist_to_target > total_length {
            for i in 1..positions.len() {
                let dir_x = target.0 - positions[i-1].0;
                let dir_y = target.1 - positions[i-1].1;
                let dir_z = target.2 - positions[i-1].2;
                let dir_len = (dir_x*dir_x + dir_y*dir_y + dir_z*dir_z).sqrt();
                if dir_len > 0.0 {
                    positions[i].0 = positions[i-1].0 + dir_x / dir_len * lengths[i-1];
                    positions[i].1 = positions[i-1].1 + dir_y / dir_len * lengths[i-1];
                    positions[i].2 = positions[i-1].2 + dir_z / dir_len * lengths[i-1];
                }
            }
            let mut rots = pose.joint_rotations.clone();
            for i in 1..chain.len() {
                let idx = chain[i];
                let dx = positions[i].0 - positions[i-1].0;
                let dy = positions[i].1 - positions[i-1].1;
                let dz = positions[i].2 - positions[i-1].2;
                let dir_len = (dx*dx + dy*dy + dz*dz).sqrt();
                if dir_len > 0.0 {
                    let rest_dir = if skeleton.joints[idx].parent_index >= 0 {
                        let base = &skeleton.joints[idx].local_transform;
                        (base.translation[0], base.translation[1], base.translation[2])
                    } else {
                        pose.root_translation
                    };
                    let rest_len = (rest_dir.0*rest_dir.0 + rest_dir.1*rest_dir.1 + rest_dir.2*rest_dir.2).sqrt();
                    let rx = if rest_len > 0.0 { rest_dir.0 / rest_len } else { 1.0 };
                    let ry = if rest_len > 0.0 { rest_dir.1 / rest_len } else { 0.0 };
                    let rz = if rest_len > 0.0 { rest_dir.2 / rest_len } else { 0.0 };
                    let dot = rx * (dx/dir_len) + ry * (dy/dir_len) + rz * (dz/dir_len);
                    let angle = dot.acos().min(1.5);
                    if angle.abs() > 0.001 {
                        let cx = ry * (dz/dir_len) - rz * (dy/dir_len);
                        let cy = rz * (dx/dir_len) - rx * (dz/dir_len);
                        let cz = rx * (dy/dir_len) - ry * (dx/dir_len);
                        let clen = (cx*cx + cy*cy + cz*cz).sqrt();
                        if clen > 0.0 {
                            rots[idx] = Quaternion::from_axis_angle((cx/clen, cy/clen, cz/clen), angle);
                        }
                    }
                }
            }
            return Pose {
                skeleton: pose.skeleton.clone(),
                joint_rotations: rots,
                root_translation: pose.root_translation,
            };
        }

        for _iter in 0..self.max_iterations {
            positions[chain.len()-1] = target;
            for i in (1..chain.len()).rev() {
                let dx = positions[i].0 - positions[i-1].0;
                let dy = positions[i].1 - positions[i-1].1;
                let dz = positions[i].2 - positions[i-1].2;
                let dir_len = (dx*dx + dy*dy + dz*dz).sqrt();
                if dir_len > 0.0 {
                    positions[i-1].0 = positions[i].0 - dx / dir_len * lengths[i-1];
                    positions[i-1].1 = positions[i].1 - dy / dir_len * lengths[i-1];
                    positions[i-1].2 = positions[i].2 - dz / dir_len * lengths[i-1];
                }
            }
            let root_pos = if skeleton.joints[chain[0]].parent_index < 0 {
                pose.root_translation
            } else {
                (0.0, 0.0, 0.0)
            };
            positions[0] = root_pos;
            for i in 1..chain.len() {
                let dx = positions[i].0 - positions[i-1].0;
                let dy = positions[i].1 - positions[i-1].1;
                let dz = positions[i].2 - positions[i-1].2;
                let dir_len = (dx*dx + dy*dy + dz*dz).sqrt();
                if dir_len > 0.0 {
                    positions[i].0 = positions[i-1].0 + dx / dir_len * lengths[i-1];
                    positions[i].1 = positions[i-1].1 + dy / dir_len * lengths[i-1];
                    positions[i].2 = positions[i-1].2 + dz / dir_len * lengths[i-1];
                }
            }
            let dx = positions[chain.len()-1].0 - target.0;
            let dy = positions[chain.len()-1].1 - target.1;
            let dz = positions[chain.len()-1].2 - target.2;
            let err = (dx*dx + dy*dy + dz*dz).sqrt();
            if err < self.tolerance {
                break;
            }
        }

        let mut rots = pose.joint_rotations.clone();
        for i in 1..chain.len() {
            let idx = chain[i];
            let dx = positions[i].0 - positions[i-1].0;
            let dy = positions[i].1 - positions[i-1].1;
            let dz = positions[i].2 - positions[i-1].2;
            let dir_len = (dx*dx + dy*dy + dz*dz).sqrt();
            if dir_len > 0.0 {
                let rest_dir = if skeleton.joints[idx].parent_index >= 0 {
                    let base = &skeleton.joints[idx].local_transform;
                    (base.translation[0], base.translation[1], base.translation[2])
                } else {
                    pose.root_translation
                };
                let rest_len = (rest_dir.0*rest_dir.0 + rest_dir.1*rest_dir.1 + rest_dir.2*rest_dir.2).sqrt();
                let rx = if rest_len > 0.0 { rest_dir.0 / rest_len } else { 1.0 };
                let ry = if rest_len > 0.0 { rest_dir.1 / rest_len } else { 0.0 };
                let rz = if rest_len > 0.0 { rest_dir.2 / rest_len } else { 0.0 };
                let dot = rx * (dx/dir_len) + ry * (dy/dir_len) + rz * (dz/dir_len);
                let angle = dot.acos().min(1.5);
                if angle.abs() > 0.001 {
                    let cx = ry * (dz/dir_len) - rz * (dy/dir_len);
                    let cy = rz * (dx/dir_len) - rx * (dz/dir_len);
                    let cz = rx * (dy/dir_len) - ry * (dx/dir_len);
                    let clen = (cx*cx + cy*cy + cz*cz).sqrt();
                    if clen > 0.0 {
                        rots[idx] = Quaternion::from_axis_angle((cx/clen, cy/clen, cz/clen), angle);
                    }
                }
            }
        }

        Pose {
            skeleton: pose.skeleton.clone(),
            joint_rotations: rots,
            root_translation: pose.root_translation,
        }
    }
}
