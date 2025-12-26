#[derive(Clone, Debug)]
pub struct Renderable {
    // For now, use simple IDs. These would refer to resources
    // managed elsewhere (e.g., in a ResourceManager).
    pub mesh: usize,
    pub shader: usize,
    pub texture: Option<usize>, // Texture might be optional
    pub is_visible: bool, // Useful flag
}

impl Renderable {
    pub fn new(mesh: usize, shader: usize, texture: Option<usize>) -> Self {
        Renderable {
            mesh,
            shader,
            texture,
            is_visible: true, // Visible by default
        }
    }

    pub fn with_visibility_flag(mut self, flag: bool) -> Self {
        self.is_visible = flag;
        self
    }

    pub fn set_visibility(&mut self, flag: bool) {
        self.is_visible = flag;
    }
} 