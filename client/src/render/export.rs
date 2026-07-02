use crate::core::skeleton::Skeleton;
use crate::animation::playback::MotionClip;

pub enum ExportFormat {
    Glb,
    Fbx,
    Onnx,
}

pub struct ExportParams {
    pub mesh: Option<MeshData>,
    pub skeleton: Option<Skeleton>,
    pub clip: Option<MotionClip>,
    pub format: ExportFormat,
    pub file_path: String,
}

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub fn export_glb(_params: &ExportParams) -> Result<(), String> {
    Err("GLB export not yet implemented".to_string())
}

pub fn export_fbx(_params: &ExportParams) -> Result<(), String> {
    Err("FBX export not yet implemented".to_string())
}

pub fn export_onnx(_params: &ExportParams) -> Result<(), String> {
    Err("ONNX export not yet implemented".to_string())
}

pub fn export_asset(params: &ExportParams) -> Result<(), String> {
    match params.format {
        ExportFormat::Glb => export_glb(params),
        ExportFormat::Fbx => export_fbx(params),
        ExportFormat::Onnx => export_onnx(params),
    }
}
