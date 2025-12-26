use gl::{ActiveTexture, BindTexture, DepthFunc, DepthMask, LEQUAL, LESS, TEXTURE0, TEXTURE_CUBE_MAP, TRUE};
use obj::{load_obj, Obj, ObjResult, Position};
use crate::graphics::mesh::{Mesh, draw, Texture};

const OBJECT: &[u8] = include_bytes!("D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\models\\skybox_cube.obj");

pub struct Skybox {
    pub mesh: Mesh<Position, u16>,
    pub texture: Texture,
}

impl Skybox {
    pub fn new() -> Self {
        let cube_object: ObjResult<Obj<Position, u16>> =
            load_obj(&OBJECT[..]);
        let cube_object = cube_object.unwrap();

        let skybox_texture_paths = [
            "D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\textures\\skyboxes\\dark\\right.png",
            "D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\textures\\skyboxes\\dark\\left.png",
            "D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\textures\\skyboxes\\dark\\top.png",
            "D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\textures\\skyboxes\\dark\\bottom.png",
            "D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\textures\\skyboxes\\dark\\front.png",
            "D:\\Core\\Workspace\\Rust\\opengl-rust-engine\\resources\\textures\\skyboxes\\dark\\back.png"
        ];

        let texture = Texture::new_cubemap(&skybox_texture_paths);
        let mesh = Mesh::<Position, u16>::new(cube_object);

        Self {
            mesh,
            texture
        }
    }

    pub fn draw(&self) {
        //  Why change depth test function?
        // https://learnopengl.com/Advanced-OpenGL/Cubemaps
        unsafe {
            DepthFunc(LEQUAL);
            // Bind skybox VAO and Texture
            ActiveTexture(TEXTURE0);
            BindTexture(TEXTURE_CUBE_MAP, self.texture.id);

            // Draw method only uses Texture2D type, so in order to draw skybox we need to bind cubemap manually
            draw(&self.mesh);

            // Good practice maybe, but skybox is same every frame, so why unbind?
            // BindTexture(TEXTURE_CUBE_MAP, 0);
            DepthMask(TRUE);
            DepthFunc(LESS);
        }
    }
}