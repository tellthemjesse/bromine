#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity(u32);

impl Entity {
    pub fn index(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Entity {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
