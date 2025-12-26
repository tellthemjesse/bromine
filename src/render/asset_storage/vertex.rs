use std::collections::hash_map::Entry;
use std::collections::HashMap;
use gl::FLOAT;
use gl::types::{GLenum, GLint, GLuint};
use num_traits::FromPrimitive;
use obj::{FromRawVertex, ObjResult};
use obj::raw::object::Polygon;

// --- Define Vertex types ---

pub enum VertexType {
    Vertex(Vertex),
    VertexUV(VertexUV),
    VertexUVN(VertexUVN)
}

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 3],
}

#[derive(Clone, Copy, Default)]
pub struct VertexUV {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone, Copy, Default)]
pub struct VertexUVN {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
    pub uv: [f32; 2],
}

// --- Implement OBJ processing for them ---

impl<I: FromPrimitive + Copy> FromRawVertex<I> for Vertex {
    fn process(
        vertices: Vec<(f32, f32, f32, f32)>,
        _: Vec<(f32, f32, f32)>,
        _: Vec<(f32, f32, f32)>,
        polygons: Vec<Polygon>,
    ) -> ObjResult<(Vec<Self>, Vec<I>)> {
        let vb = vertices
            .into_iter()
            .map(|v| Vertex {
                pos: [v.0, v.1, v.2],
            })
            .collect();
        let mut ib = Vec::with_capacity(polygons.len() * 3);
        {
            let mut map = |pi: usize| -> ObjResult<()> {
                ib.push(match I::from_usize(pi) {
                    Some(val) => val,
                    None => {
                        eprintln!("Index out of range");
                        return Ok(());
                    },
                });
                Ok(())
            };

            for polygon in polygons {
                match polygon {
                    Polygon::P(ref vec) if vec.len() == 3 => {
                        for &pi in vec {
                            map(pi)?
                        }
                    }
                    Polygon::PT(ref vec) | Polygon::PN(ref vec) if vec.len() == 3 => {
                        for &(pi, _) in vec {
                            map(pi)?
                        }
                    }
                    Polygon::PTN(ref vec) if vec.len() == 3 => {
                        for &(pi, _, _) in vec {
                            map(pi)?
                        }
                    }
                    _ => eprintln!("Model should be triangulated first to be loaded properly"),
                }
            }
        }
        Ok((vb, ib))
    }
}

impl<I: FromPrimitive + Copy> FromRawVertex<I> for VertexUV {
    fn process(
        positions: Vec<(f32, f32, f32, f32)>,
        _: Vec<(f32, f32, f32)>,
        tex_coords: Vec<(f32, f32, f32)>,
        polygons: Vec<Polygon>,
    ) -> ObjResult<(Vec<Self>, Vec<I>)> {
        let mut vb = Vec::with_capacity(polygons.len() * 3);
        let mut ib = Vec::with_capacity(polygons.len() * 3);
        {
            let mut cache = HashMap::new();
            let mut map = |pi: usize, ni: usize, ti: usize| -> ObjResult<()> {
                // Look up cache
                let index = match cache.entry((pi, ni, ti)) {
                    // Cache miss -> make new, store it on cache
                    Entry::Vacant(entry) => {
                        let p = positions[pi];
                        let t = tex_coords[ti];
                        let vertex = VertexUV {
                            pos: [p.0, p.1, p.2],
                            uv: [t.0, t.1],
                        };
                        let index = match I::from_usize(vb.len()) {
                            Some(val) => val,
                            None => {
                                eprintln!("Index out of range");
                                return Ok(());
                            },
                        };
                        vb.push(vertex);
                        entry.insert(index);
                        index
                    }
                    // Cache hit -> use it
                    Entry::Occupied(entry) => *entry.get(),
                };
                ib.push(index);
                Ok(())
            };

            for polygon in polygons {
                match polygon {
                    Polygon::P(_) | Polygon::PT(_) => eprintln!(
                        "Tried to extract normal data which are not contained in the model"
                    ),
                    Polygon::PTN(ref vec) if vec.len() == 3 => {
                        for &(pi, ni, ti) in vec {
                            map(pi, ni, ti)?;
                        }
                    }
                    _ => eprintln!(
                        "Model should be triangulated first to be loaded properly"
                    ),
                }
            }
        }
        vb.shrink_to_fit();
        Ok((vb, ib))
    }
}

impl<I: FromPrimitive + Copy> FromRawVertex<I> for VertexUVN {
    fn process(
        positions: Vec<(f32, f32, f32, f32)>,
        normals: Vec<(f32, f32, f32)>,
        tex_coords: Vec<(f32, f32, f32)>,
        polygons: Vec<Polygon>,
    ) -> ObjResult<(Vec<Self>, Vec<I>)> {
        let mut vb = Vec::with_capacity(polygons.len() * 3);
        let mut ib = Vec::with_capacity(polygons.len() * 3);
        {
            let mut cache = HashMap::new();
            let mut map = |pi: usize, ni: usize, ti: usize| -> ObjResult<()> {
                // Look up cache
                let index = match cache.entry((pi, ni, ti)) {
                    // Cache miss -> make new, store it on cache
                    Entry::Vacant(entry) => {
                        let p = positions[pi];
                        let n = normals[ni];
                        let t = tex_coords[ti];
                        let vertex = VertexUVN {
                            pos: [p.0, p.1, p.2],
                            norm: [n.0, n.1, n.2],
                            uv: [t.0, t.1],
                        };
                        let index = match I::from_usize(vb.len()) {
                            Some(val) => val,
                            None => {
                                eprintln!("Index out of range");
                                return Ok(());
                            },
                        };
                        vb.push(vertex);
                        entry.insert(index);
                        index
                    }
                    // Cache hit -> use it
                    Entry::Occupied(entry) => *entry.get(),
                };
                ib.push(index);
                Ok(())
            };

            for polygon in polygons {
                match polygon {
                    Polygon::P(_) => eprintln!("Tried to extract normal and texture data which are not contained in the model"),
                    Polygon::PT(_) => eprintln!("Tried to extract normal data which are not contained in the model"),
                    Polygon::PN(_) => eprintln!("Tried to extract texture data which are not contained in the model"),
                    Polygon::PTN(ref vec) if vec.len() == 3 => {
                        for &(pi, ti, ni) in vec { map(pi, ni, ti)? }
                    }
                    _ => eprintln!("Model should be triangulated first to be loaded properly")
                }
            }
        }
        vb.shrink_to_fit();
        Ok((vb, ib))
    }
}

// --- Define memory layout for OpenGL buffer binding ---

pub struct GLVertexAttribute {
    location: GLuint,
    size: GLint,
    types: GLenum,
}

pub struct GLVertexFormat {
    attributes: Vec<GLVertexAttribute>,
}

impl GLVertexAttribute {
    fn offset_of(&self) -> usize {
        match self.types {
            FLOAT => {
                self.location as usize * size_of::<f32>() * self.size as usize
            }
            _ => todo!()
        }
    }
}

impl GLVertexFormat {
    pub fn new(attributes: Vec<GLVertexAttribute>) -> Self {
        GLVertexFormat {
            attributes,
        }
    }
}

// --- Implement memory layout for each Vertex type ---

pub trait GLVertexLayout {
    fn layout() -> GLVertexFormat;
}

impl GLVertexLayout for Vertex {
    fn layout() -> GLVertexFormat {
        GLVertexFormat::new(vec![GLVertexAttribute {
            location: 0,
            size: 3,
            types: FLOAT,
        }])
    }
}

impl GLVertexLayout for VertexUV {
    fn layout() -> GLVertexFormat {
        GLVertexFormat::new(vec![
            GLVertexAttribute {
                location: 0,
                size: 3,
                types: FLOAT,
            },
            GLVertexAttribute {
                location: 1,
                size: 2,
                types: FLOAT,
            }
        ])
    }
}

impl GLVertexLayout for VertexUVN {
    fn layout() -> GLVertexFormat {
        GLVertexFormat::new(vec![
            GLVertexAttribute {
                location: 0,
                size: 3,
                types: FLOAT,
            },
            GLVertexAttribute {
                location: 1,
                size: 3,
                types: FLOAT,
            },
            GLVertexAttribute {
                location: 2,
                size: 2,
                types: FLOAT
            }
        ])
    }
}