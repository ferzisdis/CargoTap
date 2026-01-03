use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    device::Device,
    image::view::ImageView,
    image::{Image, ImageUsage},
    instance::Instance,
    pipeline::graphics::viewport::Viewport,
    pipeline::{GraphicsPipeline, PipelineLayout},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, acquire_next_image},
    sync::{self, GpuFuture},
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use super::pipeline;

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub attachment_image_views: Vec<Arc<ImageView>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub text_pipeline: Arc<GraphicsPipeline>,
    pub text_pipeline_layout: Arc<PipelineLayout>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

/// Creates a new swapchain for the given window surface
pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
    window_size: winit::dpi::PhysicalSize<u32>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    // Querying the capabilities of the surface. When we create the swapchain we can only
    // pass values that are allowed by the capabilities.
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    // Choosing the internal format that the images will have.
    let (image_format, _) = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0];

    // Please take a look at the docs for the meaning of the parameters we didn't mention.
    Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            // Some drivers report an `min_image_count` of 1, but fullscreen mode requires
            // at least 2. Therefore we must ensure the count is at least 2, otherwise the
            // program would crash when entering fullscreen mode on those drivers.
            min_image_count: surface_capabilities.min_image_count.max(2),

            image_format,

            // The size of the window, only used to initially setup the swapchain.
            //
            // NOTE:
            // On some drivers the swapchain extent is specified by
            // `surface_capabilities.current_extent` and the swapchain size must use this
            // extent. This extent is always the same as the window size.
            //
            // However, other drivers don't specify a value, i.e.
            // `surface_capabilities.current_extent` is `None`. These drivers will allow
            // anything, but the only sensible value is the window size.
            //
            // Both of these cases need the swapchain to use the window size, so we just
            // use that.
            image_extent: window_size.into(),

            image_usage: ImageUsage::COLOR_ATTACHMENT,

            // The alpha mode indicates how the alpha value of the final image will behave.
            // For example, you can choose whether the window will be opaque or
            // transparent.
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .unwrap(),

            ..Default::default()
        },
    )
    .unwrap()
}

/// This function is called once during initialization, then again whenever the window is resized.
pub fn window_size_dependent_setup(images: &[Arc<Image>]) -> Vec<Arc<ImageView>> {
    images
        .iter()
        .map(|image| ImageView::new_default(image.clone()).unwrap())
        .collect::<Vec<_>>()
}

/// Creates a complete render context with window, swapchain, and pipelines
pub fn create_render_context(
    event_loop: &ActiveEventLoop,
    instance: &Arc<Instance>,
    device: &Arc<Device>,
) -> RenderContext {
    // Create window
    let window = Arc::new(
        event_loop
            .create_window(Window::default_attributes())
            .unwrap(),
    );
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();
    let window_size = window.inner_size();

    // Create swapchain
    let (swapchain, images) = create_swapchain(device.clone(), surface, window_size);

    // Create image views
    let attachment_image_views = window_size_dependent_setup(&images);

    // Create graphics pipeline
    let pipeline = pipeline::create_graphics_pipeline(device.clone(), swapchain.clone());

    // Create text pipeline
    let (text_pipeline, text_pipeline_layout) =
        pipeline::create_text_pipeline(device.clone(), swapchain.clone());

    // Create viewport
    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window_size.into(),
        depth_range: 0.0..=1.0,
    };

    // Initialize frame synchronization
    let previous_frame_end = Some(sync::now(device.clone()).boxed());

    RenderContext {
        window,
        swapchain,
        attachment_image_views,
        pipeline,
        text_pipeline,
        text_pipeline_layout,
        viewport,
        recreate_swapchain: false,
        previous_frame_end,
    }
}

/// Recreates the swapchain when the window is resized
pub fn recreate_swapchain(rcx: &mut RenderContext, window_size: winit::dpi::PhysicalSize<u32>) {
    let (new_swapchain, new_images) = rcx
        .swapchain
        .recreate(SwapchainCreateInfo {
            image_extent: window_size.into(),
            ..rcx.swapchain.create_info()
        })
        .expect("failed to recreate swapchain");

    rcx.swapchain = new_swapchain;

    // Now that we have new swapchain images, we must create new image views from
    // them as well.
    rcx.attachment_image_views = window_size_dependent_setup(&new_images);

    rcx.viewport.extent = window_size.into();

    rcx.recreate_swapchain = false;
}

/// Acquires the next image from the swapchain
pub fn acquire_swapchain_image(
    rcx: &RenderContext,
) -> Result<(u32, bool, vulkano::swapchain::SwapchainAcquireFuture), VulkanError> {
    acquire_next_image(rcx.swapchain.clone(), None).map_err(Validated::unwrap)
}
