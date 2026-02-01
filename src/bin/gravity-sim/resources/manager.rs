use crate::graphics::mesh::{Mesh, Texture}; // Import Mesh
use crate::opengl_backend::shader::Program; // Re-use Program struct for shader ID?
use gl::types::{GLsizei, GLuint};
use obj::{Position, TexturedVertex};
use std::any::{Any, TypeId};
use std::fmt::Debug;

// The Resource Manager itself
#[derive(Default)] // Make it debuggable and easy to default initialize
pub struct ResourceManager {
    // Store full Mesh objects
    // TODO: Make this generic later, or use dyn Any to store different Mesh types
    meshes: Vec<Mesh<TexturedVertex, u16>>, // Store the specific Mesh type for now
    shaders: Vec<Program>,                  // Using Program struct directly for simplicity
    textures: Vec<Texture>,                 // Using Texture struct directly for simplicity

                                            // TODO: Maybe use HashMaps for named resources later?
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager::default()
    }

    // --- Mesh Management ---
    // Takes ownership of the Mesh object
    pub fn add_mesh(&mut self, mesh: Mesh<TexturedVertex, u16>) -> usize {
        let id = self.meshes.len();
        self.meshes.push(mesh);
        id
    }

    // Returns immutable reference to the Mesh
    pub fn get_mesh(&self, id: usize) -> Option<&Mesh<TexturedVertex, u16>> {
        self.meshes.get(id)
    }

    // --- Shader Management ---
    pub fn add_shader(&mut self, program: Program) -> usize {
        let id = self.shaders.len();
        self.shaders.push(program);
        id
    }

    pub fn get_shader(&self, id: usize) -> Option<&Program> {
        self.shaders.get(id)
    }

    // --- Texture Management ---
    pub fn add_texture(&mut self, texture: Texture) -> usize {
        let id = self.textures.len();
        self.textures.push(texture);
        id
    }

    pub fn get_texture(&self, id: usize) -> Option<&Texture> {
        self.textures.get(id)
    }

    // TODO: Add methods to load resources from files/bytes
    // e.g., load_mesh_from_obj_bytes, load_shader_from_strings, load_texture_from_path
}

// Trait for erased mesh type that can be stored in a Box
pub trait AnyMesh: Debug {
    fn as_any(&self) -> &dyn Any;
    fn vao(&self) -> GLuint;
    fn indices_len(&self) -> GLsizei;
}

// Implement AnyMesh for our Mesh types
impl<V: 'static + Debug, I: 'static + Debug> AnyMesh for Mesh<V, I> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn vao(&self) -> GLuint {
        self.vao
    }

    fn indices_len(&self) -> GLsizei {
        self.indices.len() as GLsizei
    }
}

// New Resource Manager that supports different vertex types
#[derive(Default)]
pub struct TypeErasedResourceMgr {
    meshes: Vec<Box<dyn AnyMesh>>, // Store mesh objects through trait object
    shaders: Vec<Program>,
    textures: Vec<Texture>,

    // Keep track of mesh types for downcasting
    mesh_types: Vec<TypeId>,
}

impl Debug for TypeErasedResourceMgr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no debug for resource mgr")
    }
}

impl TypeErasedResourceMgr {
    pub fn new() -> Self {
        TypeErasedResourceMgr::default()
    }

    // Add any mesh type that implements AnyMesh (which all our Mesh<V,I> do)
    pub fn add_mesh<V: 'static + Debug, I: 'static + Debug>(&mut self, mesh: Mesh<V, I>) -> usize {
        let id = self.meshes.len();
        let type_id = TypeId::of::<Mesh<V, I>>();
        self.mesh_types.push(type_id);
        self.meshes.push(Box::new(mesh));
        id
    }

    // Try to get a mesh as its concrete type
    pub fn get_mesh<V: 'static + Debug, I: 'static + Debug>(
        &self,
        id: usize,
    ) -> Option<&Mesh<V, I>> {
        if let Some(mesh) = self.meshes.get(id) {
            // Try to downcast to the specific mesh type
            mesh.as_any().downcast_ref::<Mesh<V, I>>()
        } else {
            None
        }
    }

    // Get the mesh as a trait object if specific type isn't needed
    pub fn get_any_mesh(&self, id: usize) -> Option<&dyn AnyMesh> {
        self.meshes.get(id).map(|m| m.as_ref())
    }

    // --- Shader Management ---
    pub fn add_shader(&mut self, program: Program) -> usize {
        let id = self.shaders.len();
        self.shaders.push(program);
        id
    }

    pub fn get_shader(&self, id: usize) -> Option<&Program> {
        self.shaders.get(id)
    }

    pub fn get_shader_mut(&mut self, id: usize) -> Option<&mut Program> {
        self.shaders.get_mut(id)
    }

    // --- Texture Management ---
    pub fn add_texture(&mut self, texture: Texture) -> usize {
        let id = self.textures.len();
        self.textures.push(texture);
        id
    }

    pub fn get_texture(&self, id: usize) -> Option<&Texture> {
        self.textures.get(id)
    }

    pub fn create_curvature_grid(size: u16, divisions: u16) -> Mesh<Position, u16> {
        let mut vertices = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        let step = size as f32 / divisions as f32;

        // Create grid vertices
        for x in 0..=divisions {
            for z in 0..=divisions {
                vertices.push(Position {
                    position: [
                        x as f32 * step - size as f32 / 2.0,
                        0.0,
                        z as f32 * step - size as f32 / 2.0,
                    ],
                });
            }
        }

        for row in 0..divisions {
            for col in 0..divisions {
                let i = row * (divisions + 1) + col;
                indices.push(i); // Horizontal line
                indices.push(i + 1);
                indices.push(i); // Vertical line
                indices.push(i + divisions + 1);
            }
        }

        Mesh::<Position, u16>::new_from_source(vertices, indices)
    }
}
