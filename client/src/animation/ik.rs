use crate::core::skeleton::{Pose, Skeleton};

pub struct FABRIKSolver {
    pub tolerance: f32,
    pub max_iterations: u32,
}

impl FABRIKSolver {
    pub fn new() -> Self {
        Self {
            tolerance: 0.01,
            max_iterations: 20,
        }
    }

    pub fn solve(
        &self,
        _skeleton: &Skeleton,
        pose: &Pose,
        _target: (f32, f32, f32),
        _chain: &[usize],
    ) -> Pose {
        pose.clone()
    }
}
