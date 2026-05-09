//! Declares [`Element`] trait. Provides [`Primitive`] enum for drawing

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u32)]
/// Primitive type used for drawing
pub enum Primitive {
    Points = gl::POINTS,
    Lines = gl::LINES,
    LineLoop = gl::LINE_LOOP,
    LineStrip = gl::LINE_STRIP,
    LineStripAdjency = gl::LINE_STRIP_ADJACENCY,
    Triangles = gl::TRIANGLES,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
}

impl TryFrom<u32> for Primitive {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            gl::POINTS => Ok(Self::Points),
            gl::LINES => Ok(Self::Lines),
            gl::LINE_LOOP => Ok(Self::LineLoop),
            gl::LINE_STRIP => Ok(Self::LineStrip),
            gl::LINE_STRIP_ADJACENCY => Ok(Self::LineStripAdjency),
            gl::TRIANGLES => Ok(Self::Triangles),
            gl::TRIANGLE_STRIP => Ok(Self::TriangleStrip),
            gl::TRIANGLE_FAN => Ok(Self::TriangleFan),
            _ => Err(anyhow!("invalid enum")),
        }
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
}

/// Indices type used for drawing
pub trait Element: private::Sealed {
    fn repr() -> u32;
}

impl Element for u8 {
    fn repr() -> u32 {
        gl::UNSIGNED_BYTE
    }
}

impl Element for u16 {
    fn repr() -> u32 {
        gl::UNSIGNED_SHORT
    }
}

impl Element for u32 {
    fn repr() -> u32 {
        gl::UNSIGNED_INT
    }
}
