use crate::animation::blend::crossfade;
use crate::core::math::Quaternion;
use crate::core::skeleton::{Pose, Skeleton};

pub struct MotionClip {
    pub skeleton: Skeleton,
    pub frames: Vec<Vec<Quaternion>>,
    pub root_positions: Vec<(f32, f32, f32)>,
    pub fps: f32,
    pub loop_: bool,
}

impl MotionClip {
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn duration(&self) -> f32 {
        if self.frame_count() == 0 {
            return 0.0;
        }
        self.frame_count() as f32 / self.fps
    }

    pub fn get_pose(&self, frame_index: usize) -> Pose {
        let idx = frame_index.min(self.frame_count().saturating_sub(1));
        Pose {
            skeleton: self.skeleton.clone(),
            joint_rotations: self.frames[idx].clone(),
            root_translation: self
                .root_positions
                .get(idx)
                .copied()
                .unwrap_or((0.0, 0.0, 0.0)),
        }
    }

    pub fn sample(&self, time: f32) -> Pose {
        if self.frame_count() == 0 {
            return Pose::new(&self.skeleton);
        }
        let mut t = time;
        if self.loop_ {
            let d = self.duration();
            if d > 0.0 {
                t = t % d;
            }
        }
        let d = self.duration();
        if d <= 0.0 {
            return Pose::new(&self.skeleton);
        }
        let n = self.frame_count();
        if n < 2 {
            let idx = 0;
            return Pose {
                skeleton: self.skeleton.clone(),
                joint_rotations: self.frames[idx].clone(),
                root_translation: self
                    .root_positions
                    .get(idx)
                    .copied()
                    .unwrap_or((0.0, 0.0, 0.0)),
            };
        }
        let normalized = t / d;
        let idx = (normalized * (n - 1) as f32) as usize;
        let frac = normalized * (n - 1) as f32 - idx as f32;
        let idx = idx.min(n - 2);
        let a = &self.frames[idx];
        let b = &self.frames[idx + 1];
        let rotations: Vec<Quaternion> = a
            .iter()
            .zip(b.iter())
            .map(|(qa, qb)| qa.slerp(*qb, frac))
            .collect();
        let root_a = self
            .root_positions
            .get(idx)
            .copied()
            .unwrap_or((0.0, 0.0, 0.0));
        let root_b = self
            .root_positions
            .get(idx + 1)
            .copied()
            .unwrap_or((0.0, 0.0, 0.0));
        let root_translation = (
            root_a.0 + (root_b.0 - root_a.0) * frac,
            root_a.1 + (root_b.1 - root_a.1) * frac,
            root_a.2 + (root_b.2 - root_a.2) * frac,
        );
        Pose {
            skeleton: self.skeleton.clone(),
            joint_rotations: rotations,
            root_translation,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = self.skeleton.validate();
        if self.frame_count() == 0 {
            errors.push("Motion clip has zero frames".to_string());
            return errors;
        }
        let jc = self.skeleton.joint_count();
        for (i, frame) in self.frames.iter().enumerate() {
            if frame.len() != jc {
                errors.push(format!(
                    "Frame {} has {} joint rotations, expected {}",
                    i,
                    frame.len(),
                    jc
                ));
            }
        }
        if !self.root_positions.is_empty() && self.root_positions.len() != self.frame_count() {
            errors.push(format!(
                "root_positions count {} != frame count {}",
                self.root_positions.len(),
                self.frame_count()
            ));
        }
        errors
    }
}

pub struct Animator {
    pub clip: Option<MotionClip>,
    pub time: f32,
    pub speed: f32,
    pub playing: bool,
    pub crossfade_clip: Option<MotionClip>,
    pub crossfade_duration: f32,
    pub crossfade_elapsed: f32,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            clip: None,
            time: 0.0,
            speed: 1.0,
            playing: true,
            crossfade_clip: None,
            crossfade_duration: 0.2,
            crossfade_elapsed: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.playing {
            return;
        }
        if self.crossfade_clip.is_some() {
            self.crossfade_elapsed += dt;
            if self.crossfade_elapsed >= self.crossfade_duration {
                self.clip = self.crossfade_clip.take();
                self.time = 0.0;
                self.crossfade_elapsed = 0.0;
            }
        }
        self.time += dt * self.speed;
    }

    pub fn current_pose(&self) -> Pose {
        let base = match &self.clip {
            Some(clip) => clip.sample(self.time),
            None => Pose::new(
                &Skeleton {
                    name: String::new(),
                    joints: Vec::new(),
                },
            ),
        };
        if let Some(ref xfade_clip) = self.crossfade_clip {
            let target = xfade_clip.sample(self.time);
            let t = (self.crossfade_elapsed / self.crossfade_duration).min(1.0);
            return crossfade(&base, &target, t);
        }
        base
    }

    pub fn play(&mut self, clip: MotionClip) {
        if self.clip.is_some() {
            self.crossfade_clip = Some(clip);
            self.crossfade_elapsed = 0.0;
        } else {
            self.clip = Some(clip);
            self.time = 0.0;
            self.playing = true;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn resume(&mut self) {
        self.playing = true;
    }
}
