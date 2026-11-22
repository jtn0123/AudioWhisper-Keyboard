//! Vello Hybrid over wgpu/Vulkan. The NDK boundary supplies a live, owned window surface.
use crate::{raster::Raster, scene::Scene};
use std::time::Instant;

pub struct Hybrid {
    // The surface drops before the NDK boundary releases the native window.
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: vello_hybrid::Renderer,
    raster: Raster,
    config: wgpu::SurfaceConfiguration,
    pub adapter: String,
}
impl std::fmt::Debug for Hybrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hybrid").finish_non_exhaustive()
    }
}

impl Hybrid {
    pub fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u16,
        height: u16,
        fonts: Vec<crate::text::FontBlob>,
    ) -> Result<Self, String> {
        pollster::block_on(Self::create(instance, surface, width, height, fonts))
    }

    async fn create(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u16,
        height: u16,
        fonts: Vec<crate::text::FontBlob>,
    ) -> Result<Self, String> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| "Vulkan adapter unavailable")?;
        let name = format!("{:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("aw-hybrid"),
                ..Default::default()
            })
            .await
            .map_err(|_| "Vulkan device unavailable")?;
        let mut config = surface
            .get_default_config(&adapter, width as u32, height as u32)
            .ok_or("Vulkan surface configuration unavailable")?;
        if let Some(format) = surface
            .get_capabilities(&adapter)
            .formats
            .into_iter()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
        {
            config.format = format;
        }
        config.present_mode = wgpu::PresentMode::Fifo;
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);
        let (renderer, resources) = vello_hybrid::Renderer::new(
            &device,
            &vello_hybrid::RenderTargetConfig {
                format: config.format,
                width: width as u32,
                height: height as u32,
            },
        );
        Ok(Self {
            surface,
            device,
            queue,
            renderer,
            raster: Raster::hybrid(width, height, fonts, resources),
            config,
            adapter: name,
        })
    }

    /// Returns CPU scene encoding time and submission-to-completion wall time, in nanoseconds.
    /// Completion includes the driver wait; it is not a GPU timestamp-query measurement.
    pub fn draw(&mut self, scene: &Scene) -> Result<(u128, u128), String> {
        let start = Instant::now();
        self.raster.draw(scene, scene.buffer_rect())?;
        let scene_encoded = start.elapsed();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => {
                self.surface.configure(&self.device, &self.config);
                return Err("Vulkan frame acquisition failed".into());
            }
        };
        let view = frame.texture.create_view(&Default::default());
        let commands_start = Instant::now();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let (context, resources) = self.raster.hybrid_parts().ok_or("wrong renderer backend")?;
        self.renderer
            .render(
                context,
                resources,
                &self.device,
                &self.queue,
                &mut encoder,
                &vello_hybrid::RenderSize {
                    width: scene.width as u32,
                    height: scene.buffer_height() as u32,
                },
                &view,
                &vello_hybrid::TextureBindings::new(),
            )
            .map_err(|_| "Vello Hybrid rendering failed")?;
        let commands = encoder.finish();
        let encoded = (scene_encoded + commands_start.elapsed()).as_nanos();
        let submitted = Instant::now();
        self.queue.submit([commands]);
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| "Vulkan completion failed")?;
        let complete = submitted.elapsed().as_nanos();
        frame.present();
        Ok((encoded, complete))
    }
}
