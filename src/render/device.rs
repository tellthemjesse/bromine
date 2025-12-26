pub enum GraphicsApi { OpenGL, Vulkan, DX12 }

pub trait RenderBacked { }

pub struct RenderDevice {
    backend: Box<dyn RenderBacked>
}
