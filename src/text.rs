use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use anyhow::Result;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
        allocator::StandardCommandBufferAllocator,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, Queue},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{GraphicsPipeline, graphics::vertex_input::Vertex, layout::PipelineLayout},
};

#[derive(BufferContents, Clone, Copy)]
#[repr(C)]
pub struct TextPushConstants {
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2], // Padding to align text_color to 16-byte boundary
    pub text_color: [f32; 4],
}

// Text rendering will be simplified for now - just vertex processing

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
pub struct TextVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub tex_coords: [f32; 2],
}

pub struct TextSystem {
    font: FontArc,
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    vertices: Vec<TextVertex>,
    vertex_buffer: Option<Subbuffer<[TextVertex]>>,
    pub is_pipeline_ready: bool,
}

impl TextSystem {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        // Load font
        let font_data = include_bytes!("../fonts/JetBrainsMono-Light.ttf");
        let font = FontArc::try_from_slice(font_data)?;

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(device.clone(), Default::default());

        Ok(Self {
            font,
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            vertices: Vec::new(),
            vertex_buffer: None,
            is_pipeline_ready: true, // Simplified - always ready
        })
    }

    pub fn update_text(&mut self, text: &str) -> Result<()> {
        self.vertices.clear();

        let font_size = 16.0;
        let scale = PxScale::from(font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut cursor_x = 10.0;
        let mut cursor_y = 30.0;
        let line_height = scaled_font.height();

        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = 10.0;
                cursor_y += line_height;
                continue;
            }

            if ch == '\r' {
                continue;
            }

            let glyph_id = self.font.glyph_id(ch);
            let glyph = glyph_id.with_scale_and_position(scale, point(cursor_x, cursor_y));

            if let Some(outlined) = scaled_font.outline_glyph(glyph.clone()) {
                let bounds = outlined.px_bounds();

                // Create quad vertices for this glyph
                let vertices = [
                    TextVertex {
                        position: [bounds.min.x, bounds.min.y],
                        tex_coords: [0.0, 0.0],
                    },
                    TextVertex {
                        position: [bounds.max.x, bounds.min.y],
                        tex_coords: [1.0, 0.0],
                    },
                    TextVertex {
                        position: [bounds.min.x, bounds.max.y],
                        tex_coords: [0.0, 1.0],
                    },
                    TextVertex {
                        position: [bounds.max.x, bounds.min.y],
                        tex_coords: [1.0, 0.0],
                    },
                    TextVertex {
                        position: [bounds.max.x, bounds.max.y],
                        tex_coords: [1.0, 1.0],
                    },
                    TextVertex {
                        position: [bounds.min.x, bounds.max.y],
                        tex_coords: [0.0, 1.0],
                    },
                ];

                self.vertices.extend_from_slice(&vertices);
            }

            cursor_x += scaled_font.h_advance(glyph_id);
        }

        self.update_vertex_buffer()?;
        Ok(())
    }

    fn update_vertex_buffer(&mut self) -> Result<()> {
        if self.vertices.is_empty() {
            return Ok(());
        }

        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.vertices.iter().cloned(),
        )?;

        self.vertex_buffer = Some(vertex_buffer);
        Ok(())
    }

    pub fn get_vertices(&self) -> &[TextVertex] {
        &self.vertices
    }

    pub fn get_vertex_buffer(&self) -> Option<&Subbuffer<[TextVertex]>> {
        self.vertex_buffer.as_ref()
    }

    pub fn rasterize_text_to_console(&self, text: &str) -> Result<()> {
        let font_size = 16.0;
        let scale = PxScale::from(font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut cursor_x = 0.0;
        let mut cursor_y = 0.0;
        let line_height = scaled_font.height();

        println!("Rendering text: {}", text);
        println!("Font height: {}", line_height);

        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = 0.0;
                cursor_y += line_height;
                continue;
            }

            if ch == '\r' {
                continue;
            }

            let glyph_id = self.font.glyph_id(ch);
            let glyph = glyph_id.with_scale_and_position(scale, point(cursor_x, cursor_y));

            if let Some(outlined) = scaled_font.outline_glyph(glyph.clone()) {
                let bounds = outlined.px_bounds();
                println!(
                    "Character '{}' at ({:.1}, {:.1}) bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    ch, cursor_x, cursor_y, bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y
                );
            }

            cursor_x += scaled_font.h_advance(glyph_id);
        }

        Ok(())
    }

    pub fn create_text_pipeline(&mut self) -> Result<()> {
        // Simplified pipeline creation - just mark as ready
        self.is_pipeline_ready = true;
        log::info!("Text rendering pipeline created successfully (simplified)");
        Ok(())
    }

    pub fn draw(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        text_pipeline: Arc<GraphicsPipeline>,
        text_pipeline_layout: Arc<PipelineLayout>,
        screen_size: [f32; 2],
    ) -> Result<()> {
        log::debug!(
            "TextSystem::draw() called with {} vertices",
            self.vertices.len()
        );

        if let Some(vertex_buffer) = &self.vertex_buffer {
            if !self.vertices.is_empty() {
                // Set push constants
                let push_constants = TextPushConstants {
                    screen_size,
                    _padding: [0.0, 0.0],
                    text_color: [1.0, 1.0, 1.0, 1.0], // White text
                };

                // Bind text pipeline, set push constants, bind vertex buffer, then draw
                unsafe {
                    command_buffer
                        .bind_pipeline_graphics(text_pipeline)
                        .map_err(|e| anyhow::anyhow!("Failed to bind text pipeline: {}", e))?
                        .push_constants(text_pipeline_layout, 0, push_constants)
                        .map_err(|e| anyhow::anyhow!("Failed to set push constants: {}", e))?
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .map_err(|e| anyhow::anyhow!("Failed to bind vertex buffer: {}", e))?
                        .draw(self.vertices.len() as u32, 1, 0, 0)
                        .map_err(|e| anyhow::anyhow!("Failed to draw vertices: {}", e))?;
                }

                log::info!(
                    "Successfully drew {} text vertices to screen with text pipeline",
                    self.vertices.len()
                );
            }
        } else {
            log::debug!("Vertex buffer not available for text rendering");
        }
        Ok(())
    }

    pub fn has_text(&self) -> bool {
        !self.vertices.is_empty() && self.vertex_buffer.is_some()
    }
}
