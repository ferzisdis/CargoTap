use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract,
        allocator::StandardCommandBufferAllocator,
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
    },
    device::{Device, Queue},
    format::Format,
    image::{
        Image, ImageCreateInfo, ImageType, ImageUsage,
        sampler::{Sampler, SamplerCreateInfo},
        view::ImageView,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{GraphicsPipeline, graphics::vertex_input::Vertex, layout::PipelineLayout},
    sync::GpuFuture,
};

#[derive(BufferContents, Clone, Copy)]
#[repr(C)]
pub struct TextPushConstants {
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2], // Padding to align text_color to 16-byte boundary
    pub text_color: [f32; 4],
}

#[derive(Clone, Copy)]
pub struct TextRenderSettings {
    pub color: [f32; 4],
    pub font_size: f32,
    pub position: [f32; 2],
}

impl Default for TextRenderSettings {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0], // White
            font_size: 32.0,
            position: [10.0, 30.0],
        }
    }
}

#[derive(Clone, Copy)]
struct GlyphInfo {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    size: [f32; 2],
    bearing: [f32; 2],
    advance: f32,
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
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    vertices: Vec<TextVertex>,
    vertex_buffer: Option<Subbuffer<[TextVertex]>>,
    pub is_pipeline_ready: bool,

    // Texture atlas
    atlas_texture: Option<Arc<ImageView>>,
    atlas_sampler: Option<Arc<Sampler>>,
    glyph_infos: HashMap<char, GlyphInfo>,
    descriptor_set: Option<Arc<DescriptorSet>>,
    current_settings: TextRenderSettings,
}

impl TextSystem {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        settings: TextRenderSettings,
    ) -> Result<Self> {
        // Load font
        let font_data = include_bytes!("../fonts/JetBrainsMono-Light.ttf");
        let font = FontArc::try_from_slice(font_data)?;

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Ok(Self {
            font,
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            vertices: Vec::new(),
            vertex_buffer: None,
            is_pipeline_ready: false, // Will be ready after atlas creation

            atlas_texture: None,
            atlas_sampler: None,
            glyph_infos: HashMap::new(),
            descriptor_set: None,
            current_settings: settings,
        })
    }

    pub fn update_text_with_settings(&mut self, text: &str) -> Result<()> {
        self.vertices.clear();

        let scale = PxScale::from(self.current_settings.font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut cursor_x = self.current_settings.position[0];
        let mut cursor_y = self.current_settings.position[1];
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

            // Get glyph info from atlas
            if let Some(glyph_info) = self.glyph_infos.get(&ch) {
                let pos_x = cursor_x + glyph_info.bearing[0];
                let pos_y = cursor_y + glyph_info.bearing[1];

                // Create quad vertices for this glyph using atlas UV coordinates
                let vertices = [
                    TextVertex {
                        position: [pos_x, pos_y],
                        tex_coords: [glyph_info.uv_min[0], glyph_info.uv_min[1]],
                    },
                    TextVertex {
                        position: [pos_x + glyph_info.size[0], pos_y],
                        tex_coords: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                    },
                    TextVertex {
                        position: [pos_x, pos_y + glyph_info.size[1]],
                        tex_coords: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                    },
                    TextVertex {
                        position: [pos_x + glyph_info.size[0], pos_y],
                        tex_coords: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                    },
                    TextVertex {
                        position: [pos_x + glyph_info.size[0], pos_y + glyph_info.size[1]],
                        tex_coords: [glyph_info.uv_max[0], glyph_info.uv_max[1]],
                    },
                    TextVertex {
                        position: [pos_x, pos_y + glyph_info.size[1]],
                        tex_coords: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                    },
                ];

                self.vertices.extend_from_slice(&vertices);
                cursor_x += glyph_info.advance;
            } else {
                // Fallback for missing glyphs
                let glyph_id = self.font.glyph_id(ch);
                cursor_x += scaled_font.h_advance(glyph_id);
            }
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

    pub fn create_text_atlas(&mut self, pipeline_layout: Arc<PipelineLayout>) -> Result<()> {
        // Create texture atlas
        const ATLAS_SIZE: u32 = 512;
        let font_size = self.current_settings.font_size;

        log::info!(
            "Creating font atlas with size {}x{} for font size {}",
            ATLAS_SIZE,
            ATLAS_SIZE,
            font_size
        );

        // Create atlas texture
        let atlas_image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8_UNORM, // Single channel for alpha
                extent: [ATLAS_SIZE, ATLAS_SIZE, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )?;

        let atlas_view = ImageView::new_default(atlas_image)?;

        // Create sampler
        let sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo::simple_repeat_linear(),
        )?;

        // Rasterize glyphs to atlas
        let mut atlas_data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE) as usize];
        let scale = PxScale::from(font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut current_x = 0;
        let mut current_y = 0;
        let mut row_height = 0;

        // ASCII printable characters
        for ch in (32u8..127u8).map(|c| c as char) {
            let glyph_id = self.font.glyph_id(ch);
            let glyph = glyph_id.with_scale(scale);

            if let Some(outlined) = scaled_font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                let width = bounds.width() as u32;
                let height = bounds.height() as u32;

                // Check if we need to move to next row
                if current_x + width > ATLAS_SIZE {
                    current_x = 0;
                    current_y += row_height;
                    row_height = 0;
                }

                // Check if we have space
                if current_y + height > ATLAS_SIZE {
                    break;
                }

                // Rasterize glyph
                outlined.draw(|x, y, coverage| {
                    let atlas_x = current_x + x;
                    let atlas_y = current_y + y;
                    if atlas_x < ATLAS_SIZE && atlas_y < ATLAS_SIZE {
                        let index = (atlas_y * ATLAS_SIZE + atlas_x) as usize;
                        atlas_data[index] = (coverage * 255.0) as u8;
                    }
                });

                // Store glyph info
                let glyph_info = GlyphInfo {
                    uv_min: [
                        current_x as f32 / ATLAS_SIZE as f32,
                        current_y as f32 / ATLAS_SIZE as f32,
                    ],
                    uv_max: [
                        (current_x + width) as f32 / ATLAS_SIZE as f32,
                        (current_y + height) as f32 / ATLAS_SIZE as f32,
                    ],
                    size: [bounds.width(), bounds.height()],
                    bearing: [bounds.min.x, bounds.min.y],
                    advance: scaled_font.h_advance(glyph_id),
                };

                self.glyph_infos.insert(ch, glyph_info);

                current_x += width + 1; // Add 1 pixel padding
                row_height = row_height.max(height + 1);
            }
        }

        // Upload atlas data to GPU
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )?;

        builder.copy_buffer_to_image(
            vulkano::command_buffer::CopyBufferToImageInfo::buffer_image(
                Buffer::from_iter(
                    self.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    atlas_data,
                )?,
                atlas_view.image().clone(),
            ),
        )?;

        let command_buffer = builder.build()?;
        command_buffer
            .execute(self.queue.clone())?
            .then_signal_fence_and_flush()?
            .wait(None)?;

        // Create descriptor set
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            pipeline_layout.set_layouts().get(0).unwrap().clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                atlas_view.clone(),
                sampler.clone(),
            )],
            [],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create descriptor set: {}", e))?;

        self.atlas_texture = Some(atlas_view);
        self.atlas_sampler = Some(sampler);
        self.descriptor_set = Some(descriptor_set);
        self.is_pipeline_ready = true;

        log::info!(
            "Text atlas created successfully with {} glyphs, atlas utilization: {:.1}%",
            self.glyph_infos.len(),
            (current_y as f32 / ATLAS_SIZE as f32) * 100.0
        );
        Ok(())
    }

    pub fn create_text_pipeline(&mut self) -> Result<()> {
        // This is now handled by create_text_atlas
        log::info!("Text rendering pipeline marked as ready");
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

        if let (Some(vertex_buffer), Some(descriptor_set)) =
            (&self.vertex_buffer, &self.descriptor_set)
        {
            if !self.vertices.is_empty() {
                // Set push constants
                let push_constants = TextPushConstants {
                    screen_size,
                    _padding: [0.0, 0.0],
                    text_color: self.current_settings.color,
                };

                // Bind text pipeline, descriptor set, set push constants, bind vertex buffer, then draw
                unsafe {
                    command_buffer
                        .bind_pipeline_graphics(text_pipeline)
                        .map_err(|e| anyhow::anyhow!("Failed to bind text pipeline: {}", e))?
                        .bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics,
                            text_pipeline_layout.clone(),
                            0,
                            descriptor_set.clone(),
                        )
                        .map_err(|e| anyhow::anyhow!("Failed to bind descriptor set: {}", e))?
                        .push_constants(text_pipeline_layout, 0, push_constants)
                        .map_err(|e| anyhow::anyhow!("Failed to set push constants: {}", e))?
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .map_err(|e| anyhow::anyhow!("Failed to bind vertex buffer: {}", e))?
                        .draw(self.vertices.len() as u32, 1, 0, 0)
                        .map_err(|e| anyhow::anyhow!("Failed to draw vertices: {}", e))?;
                }

                log::debug!(
                    "Successfully drew {} text vertices to screen with text pipeline",
                    self.vertices.len()
                );
            }
        } else {
            log::debug!("Vertex buffer or descriptor set not available for text rendering");
        }
        Ok(())
    }

    pub fn has_text(&self) -> bool {
        !self.vertices.is_empty() && self.vertex_buffer.is_some() && self.descriptor_set.is_some()
    }
}
