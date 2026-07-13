use std::fs;
use std::path::Path;

use crate::core::ecs::*;
use crate::core::math::{multiply_mat4, Quaternion, Transform};

pub struct Scene {
    pub world: EcsWorld,
}

impl Scene {
    pub fn new() -> Self {
        Self { world: EcsWorld::new() }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let entities: Vec<serde_json::Value> = self.world.query::<TransformComponent>().iter().map(|(id, t)| {
            let label = self.world.get::<LabelComponent>(*id).map(|l| {
                serde_json::json!({"name": l.name, "entity_type": l.entity_type})
            }).unwrap_or(serde_json::json!({}));
            let mesh = self.world.get::<MeshComponent>(*id).map(|m| {
                serde_json::json!({
                    "mesh_data": m.mesh_data,
                })
            }).unwrap_or(serde_json::json!({}));
            let material = self.world.get::<MaterialComponent>(*id).map(|m| {
                serde_json::json!({
                    "albedo": [m.albedo.0, m.albedo.1, m.albedo.2],
                    "metallic": m.metallic,
                    "roughness": m.roughness,
                    "ambient_occlusion": m.ambient_occlusion,
                })
            }).unwrap_or(serde_json::json!({}));
            serde_json::json!({
                "entity_id": id,
                "transform": {
                    "position": [t.position.0, t.position.1, t.position.2],
                    "rotation": [t.rotation.0, t.rotation.1, t.rotation.2],
                    "scale": [t.scale.0, t.scale.1, t.scale.2],
                    "parent_id": t.parent_id,
                },
                "label": label,
                "mesh": mesh,
                "material": material,
            })
        }).collect();
        let json = serde_json::json!({"entities": entities});
        fs::write(Path::new(path), serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(Path::new(path)).map_err(|e| e.to_string())?;
        let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let entities = data["entities"].as_array().ok_or("No entities in scene")?;
        for ent in entities {
            let eid = ent["entity_id"].as_u64().ok_or("Invalid entity_id")?;
            self.world.spawn_with_id(eid);
            if let Some(t) = ent["transform"].as_object() {
                let pos = t["position"].as_array().map(|a| (a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32)).unwrap_or((0.0, 0.0, 0.0));
                let rot = t["rotation"].as_array().map(|a| (a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32)).unwrap_or((0.0, 0.0, 0.0));
                let scale = t["scale"].as_array().map(|a| (a[0].as_f64().unwrap_or(1.0) as f32, a[1].as_f64().unwrap_or(1.0) as f32, a[2].as_f64().unwrap_or(1.0) as f32)).unwrap_or((1.0, 1.0, 1.0));
                let parent_id = t["parent_id"].as_u64();
                self.world.add(eid, TransformComponent { position: pos, rotation: rot, scale, parent_id });
            }
            if let Some(label) = ent["label"].as_object() {
                let name = label["name"].as_str().unwrap_or("").to_string();
                let entity_type = label["entity_type"].as_str().unwrap_or("").to_string();
                self.world.add(eid, LabelComponent { name, entity_type });
            }
            if let Some(mat) = ent["material"].as_object() {
                let albedo_arr = mat["albedo"].as_array().map(|a| (a[0].as_f64().unwrap_or(0.8) as f32, a[1].as_f64().unwrap_or(0.8) as f32, a[2].as_f64().unwrap_or(0.8) as f32)).unwrap_or((0.8, 0.8, 0.8));
                let metallic = mat["metallic"].as_f64().unwrap_or(0.0) as f32;
                let roughness = mat["roughness"].as_f64().unwrap_or(0.5) as f32;
                let ao = mat["ambient_occlusion"].as_f64().unwrap_or(1.0) as f32;
                self.world.add(eid, MaterialComponent { albedo: albedo_arr, metallic, roughness, ambient_occlusion: ao });
            }
        }
        Ok(())
    }

    pub fn spawn_primitive(&mut self, mesh_type: MeshType) -> EntityId {
        let id = self.world.spawn();
        self.world.add(id, TransformComponent::identity());
        self.world.add(id, MeshComponent::from_type(mesh_type));
        self.world.add(id, MaterialComponent {
            albedo: (0.8, 0.8, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            ambient_occlusion: 1.0,
        });
        self.world.add(id, LabelComponent {
            name: String::new(),
            entity_type: String::new(),
        });
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.world.despawn(id);
    }

    pub fn clear_all(&mut self) {
        let ids: Vec<EntityId> = self.world.query::<TransformComponent>().iter().map(|(id, _)| *id).collect();
        for id in ids {
            self.world.despawn(id);
        }
    }

    pub fn entity_count(&self) -> usize {
        self.world.query::<TransformComponent>().len()
    }

    pub fn compute_world_matrix(&self, id: EntityId) -> [f32; 16] {
        let world = &self.world;
        let local = match world.get::<TransformComponent>(id) {
            Some(t) => *t,
            None => return crate::render::static_renderer::identity_matrix(),
        };

        let rotation = Quaternion::from_euler(
            local.rotation.0,
            local.rotation.1,
            local.rotation.2,
        );
        let transform = Transform {
            translation: local.position,
            rotation,
            scale: local.scale,
        };
        let local_mat = transform.to_matrix();

        match local.parent_id {
            None => local_mat,
            Some(pid) => {
                let parent_mat = self.compute_world_matrix(pid);
                multiply_mat4(&parent_mat, &local_mat)
            }
        }
    }

    pub fn collect_render_data(&self, selected: Option<EntityId>) -> Vec<(MeshType, [f32; 16], (f32, f32, f32), f32, f32)> {
        let transforms = self.world.query::<TransformComponent>();
        let meshes = self.world.query::<MeshComponent>();
        let materials = self.world.query::<MaterialComponent>();

        let mut results = Vec::new();

        for (id, _transform) in &transforms {
            let mesh = match meshes.iter().find(|(mid, _)| *mid == *id) {
                Some((_, m)) => m,
                None => continue,
            };
            let mesh_type = match &mesh.mesh_type {
                Some(mt) => mt.clone(),
                None => continue,
            };
            let material = materials.iter().find(|(mid, _)| *mid == *id).map(|(_, m)| *m);
            let world_mat = self.compute_world_matrix(*id);
            let (r, g, b, metallic, roughness) = match material {
                Some(m) => (m.albedo.0, m.albedo.1, m.albedo.2, m.metallic, m.roughness),
                None => (0.8, 0.8, 0.8, 0.0, 0.5),
            };
            let (r, g, b) = if Some(*id) == selected {
                (r.min(1.0) * 1.5, g.min(1.0) * 1.5, b.min(1.0) * 1.5)
            } else {
                (r, g, b)
            };
            results.push((mesh_type, world_mat, (r, g, b), metallic, roughness));
        }

        results
    }
}
