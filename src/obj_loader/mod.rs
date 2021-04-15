// Modules are ways of organizing code within a crate for easy reuse and readability.
// Modules also control the privacy of items. They will be private by default.
// Modules are defined using the "mod" keyword.
use std::path::PathBuf;

#[derive(Default)]
pub struct ObjLoaderResult {
    pub vertices: Vec<f32>,
    pub indices: Vec<f32>
}

pub fn load_obj(file_path: PathBuf) -> ObjLoaderResult {
    

    ObjLoaderResult::default()
}
