use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderingAttachmentInfo, RenderingInfo,
        allocator::StandardCommandBufferAllocator,
    },
    device::{Device, Queue},
    instance::Instance,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{GraphicsPipeline, PipelineLayout},
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
    swapchain::SwapchainPresentInfo,
    sync::{self, GpuFuture},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

mod pipeline;
mod swapchain;
mod vulkan_init;

use pipeline::MyVertex;
use swapchain::RenderContext;

pub struct VulkanRenderer {
    instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    vertex_buffer: Subbuffer<[MyVertex]>,
    rcx: Option<RenderContext>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    text_system: Option<Arc<std::sync::Mutex<crate::text::TextSystem>>>,
}

impl VulkanRenderer {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let (instance, device, queue) = vulkan_init::initialize_vulkan(event_loop);

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Create vertex buffer for the triangle
        let vertices = [
            MyVertex {
                position: [-0.5, -0.25],
            },
            MyVertex {
                position: [0.0, 0.5],
            },
            MyVertex {
                position: [0.25, -0.1],
            },
        ];

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        Self {
            instance,
            device,
            queue,
            command_buffer_allocator,
            vertex_buffer,
            rcx: None,
            memory_allocator,
            text_system: None,
        }
    }

    pub fn set_text_system(&mut self, text_system: Arc<std::sync::Mutex<crate::text::TextSystem>>) {
        self.text_system = Some(text_system);
    }

    pub fn is_ready(&self) -> bool {
        self.rcx.is_some()
    }

    pub fn get_text_pipeline(&self) -> Option<Arc<GraphicsPipeline>> {
        self.rcx.as_ref().map(|rcx| rcx.text_pipeline.clone())
    }

    pub fn get_text_pipeline_layout(&self) -> Option<Arc<PipelineLayout>> {
        self.rcx
            .as_ref()
            .map(|rcx| rcx.text_pipeline_layout.clone())
    }

    pub fn get_window_size(&self) -> Option<[f32; 2]> {
        self.rcx.as_ref().map(|rcx| {
            let size = rcx.window.inner_size();
            [size.width as f32, size.height as f32]
        })
    }
}

impl ApplicationHandler for VulkanRenderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rcx = Some(swapchain::create_render_context(
            event_loop,
            &self.instance,
            &self.device,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let rcx = self.rcx.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                rcx.recreate_swapchain = true;

                // Update TextSystem with new window size
                if let Some(text_system) = &self.text_system {
                    if let Ok(mut text_system) = text_system.lock() {
                        text_system
                            .update_window_size(new_size.width as f32, new_size.height as f32);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let window_size = rcx.window.inner_size();

                // Do not draw the frame when the screen size is zero. On Windows, this can occur
                // when minimizing the application.
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }

                // It is important to call this function from time to time, otherwise resources
                // will keep accumulating and you will eventually reach an out of memory error.
                // Calling this function polls various fences in order to determine what the GPU
                // has already processed, and frees the resources that are no longer needed.
                rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();

                // Whenever the window resizes we need to recreate everything dependent on the
                // window size. In this example that includes the swapchain, the framebuffers and
                // the dynamic state viewport.
                if rcx.recreate_swapchain {
                    swapchain::recreate_swapchain(rcx, window_size);

                    // Update TextSystem with current window size after swapchain recreation
                    if let Some(text_system) = &self.text_system {
                        if let Ok(mut text_system) = text_system.lock() {
                            text_system.update_window_size(
                                window_size.width as f32,
                                window_size.height as f32,
                            );
                        }
                    }
                }

                // Acquire next image from swapchain
                let (image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_swapchain_image(rcx) {
                        Ok(r) => r,
                        Err(_) => {
                            rcx.recreate_swapchain = true;
                            return;
                        }
                    };

                if suboptimal {
                    rcx.recreate_swapchain = true;
                }

                // Record command buffer
                let mut builder = AutoCommandBufferBuilder::primary(
                    self.command_buffer_allocator.clone(),
                    self.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    .begin_rendering(RenderingInfo {
                        color_attachments: vec![Some(RenderingAttachmentInfo {
                            load_op: AttachmentLoadOp::Clear,
                            store_op: AttachmentStoreOp::Store,
                            clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
                            ..RenderingAttachmentInfo::image_view(
                                rcx.attachment_image_views[image_index as usize].clone(),
                            )
                        })],
                        ..Default::default()
                    })
                    .unwrap()
                    .set_viewport(0, [rcx.viewport.clone()].into_iter().collect())
                    .unwrap()
                    .bind_pipeline_graphics(rcx.pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, self.vertex_buffer.clone())
                    .unwrap();

                // Draw text if available
                if let Some(text_system) = &self.text_system {
                    log::debug!("Text system is available, attempting to acquire lock");
                    if let Ok(text_system) = text_system.lock() {
                        if text_system.has_text() {
                            log::debug!("Drawing text to screen {:?}", window_size);
                            if let Err(e) = text_system.draw(
                                &mut builder,
                                rcx.text_pipeline.clone(),
                                rcx.text_pipeline_layout.clone(),
                            ) {
                                log::warn!("Failed to draw text: {}", e);
                            }
                        } else {
                            log::debug!("Text system has no text to draw");
                        }
                    } else {
                        log::warn!("Failed to acquire lock on text system");
                    }
                } else {
                    log::debug!("No text system available for drawing");
                }

                builder.end_rendering().unwrap();

                let command_buffer = builder.build().unwrap();

                let future = rcx
                    .previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        self.queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            rcx.swapchain.clone(),
                            image_index,
                        ),
                    )
                    .then_signal_fence_and_flush();

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        rcx.previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        rcx.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("failed to flush future: {e}");
                        rcx.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let rcx = self.rcx.as_mut().unwrap();
        rcx.window.request_redraw();
    }
}
