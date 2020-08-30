//! Robust implementation of `GraphicsDisplay` using Google's Skia.
//! From https://github.com/jazzfool/reclutch/
//! MIT licensed
//! See also: https://github.com/ctrlcctrlv/imgui-skia-example

extern crate gl;
use {
    skia_safe as sk,
    std::collections::HashMap,
};

mod error;

/// Contains information about an existing OpenGL framebuffer.
#[derive(Debug, Clone, Copy)]
pub struct SkiaOpenGlFramebuffer {
    pub size: (i32, i32),
    pub framebuffer_id: u32,
}

/// Contains information about an existing OpenGL texture.
#[derive(Debug, Clone, Copy)]
pub struct SkiaOpenGlTexture {
    pub size: (i32, i32),
    pub mip_mapped: bool,
    pub texture_id: u32,
}

pub enum SurfaceType {
    OpenGlFramebuffer(SkiaOpenGlFramebuffer),
    OpenGlTexture(SkiaOpenGlTexture),
}

enum Resource {
    Image(sk::Image),
    Font(sk::Typeface),
}

/// Converts [`DisplayCommand`](crate::display::DisplayCommand) to immediate-mode Skia commands.
pub struct SkiaGraphicsDisplay {
    pub surface: sk::Surface,
    pub surface_type: SurfaceType,
    pub context: sk::gpu::Context,
    next_command_group_id: u64,
    resources: HashMap<u64, Resource>,
    next_resource_id: u64,
}

impl SkiaGraphicsDisplay {
    /// Creates a new [`SkiaGraphicsDisplay`](SkiaGraphicsDisplay) with the Skia OpenGL backend, drawing into an existing framebuffer.
    /// This assumes that an OpenGL context has already been set up.
    /// This also assumes that the color format is RGBA with 8-bit components.
    pub fn new_gl_framebuffer(target: &SkiaOpenGlFramebuffer) -> Result<Self, error::SkiaError> {
        let (surface, context) = Self::new_gl_framebuffer_surface(target)?;
        Ok(Self {
            surface,
            surface_type: SurfaceType::OpenGlFramebuffer(*target),
            context,
            next_command_group_id: 0,
            resources: HashMap::new(),
            next_resource_id: 0,
        })
    }

    /// Creates a new [`SkiaGraphicsDisplay`](SkiaGraphicsDisplay) with the Skia OpenGL backend, drawing into an existing texture.
    /// This assumes that an OpenGL context has already been set up.
    /// This also assumes that the color format is RGBA with 8-bit components
    pub fn new_gl_texture(target: &SkiaOpenGlTexture) -> Result<Self, error::SkiaError> {
        let (surface, context) = Self::new_gl_texture_surface(target)?;
        Ok(Self {
            surface,
            surface_type: SurfaceType::OpenGlTexture(*target),
            context,
            next_command_group_id: 0,
            resources: HashMap::new(),
            next_resource_id: 0,
        })
    }

    /// Returns the size of the underlying surface.
    pub fn size(&self) -> (i32, i32) {
        match self.surface_type {
            SurfaceType::OpenGlFramebuffer(SkiaOpenGlFramebuffer { size, .. })
            | SurfaceType::OpenGlTexture(SkiaOpenGlTexture { size, .. }) => size,
        }
    }

    fn new_gl_framebuffer_surface(
        target: &SkiaOpenGlFramebuffer,
    ) -> Result<(sk::Surface, sk::gpu::Context), error::SkiaError> {
        let mut context = Self::new_gl_context()?;

        Ok((SkiaGraphicsDisplay::new_gl_framebuffer_from_context(target, &mut context)?, context))
    }

    fn new_gl_framebuffer_from_context(
        target: &SkiaOpenGlFramebuffer,
        context: &mut sk::gpu::Context,
    ) -> Result<sk::Surface, error::SkiaError> {
        let info = sk::gpu::BackendRenderTarget::new_gl(
            target.size,
            None,
            8,
            sk::gpu::gl::FramebufferInfo { fboid: target.framebuffer_id, format: gl::RGBA8 },
        );

        Ok(sk::Surface::from_backend_render_target(
            context,
            &info,
            sk::gpu::SurfaceOrigin::BottomLeft,
            sk::ColorType::RGBA8888,
            sk::ColorSpace::new_srgb(),
            None,
        )
        .ok_or_else(|| error::SkiaError::InvalidTarget(String::from("framebuffer")))?)
    }

    fn new_gl_texture_surface(
        target: &SkiaOpenGlTexture,
    ) -> Result<(sk::Surface, sk::gpu::Context), error::SkiaError> {
        let mut context = Self::new_gl_context()?;

        Ok((SkiaGraphicsDisplay::new_gl_texture_from_context(target, &mut context)?, context))
    }

    fn new_gl_texture_from_context(
        target: &SkiaOpenGlTexture,
        context: &mut sk::gpu::Context,
    ) -> Result<sk::Surface, error::SkiaError> {
        let info = unsafe {
            sk::gpu::BackendTexture::new_gl(
                target.size,
                if target.mip_mapped { sk::gpu::MipMapped::Yes } else { sk::gpu::MipMapped::No },
                sk::gpu::gl::TextureInfo {
                    format: gl::RGBA8,
                    target: gl::TEXTURE_2D,
                    id: target.texture_id,
                },
            )
        };

        Ok(sk::Surface::from_backend_texture(
            context,
            &info,
            sk::gpu::SurfaceOrigin::BottomLeft,
            None,
            sk::ColorType::RGBA8888,
            sk::ColorSpace::new_srgb(),
            None,
        )
        .ok_or_else(|| error::SkiaError::InvalidTarget(String::from("texture")))?)
    }

    fn new_gl_context() -> Result<sk::gpu::Context, error::SkiaError> {
        sk::gpu::Context::new_gl(sk::gpu::gl::Interface::new_native())
            .ok_or(error::SkiaError::InvalidContext)
    }
}
