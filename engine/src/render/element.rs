#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u32)]
/// Primitive type used for drawing
pub enum Primitive {
    Points = gl::POINTS,
    Lines = gl::LINES,
    Triangles = gl::TRIANGLES,
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
