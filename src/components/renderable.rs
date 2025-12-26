#[derive(Clone, Debug)]
pub struct Renderable {
    pub mesh: usize,
    pub shader: usize,
    pub texture: Option<usize>,
    pub is_visible: bool,
}

impl Renderable {
    pub fn new(mesh: usize, shader: usize, texture: Option<usize>) -> Self {
        Renderable {
            mesh,
            shader,
            texture,
            is_visible: true,
        }
    }

    pub fn with_visibility_flag(mut self, flag: bool) -> Self {
        self.is_visible = flag;
        self
    }

    #[allow(unused)]
    pub fn set_visibility(&mut self, flag: bool) {
        self.is_visible = flag;
    }
}
