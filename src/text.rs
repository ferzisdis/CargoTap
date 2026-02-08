use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use anyhow::Result;
use image::{ImageBuffer, Luma};
use std::{collections::HashMap, env, sync::Arc};
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

#[derive(Debug, Clone)]
pub struct ColoredChar {
    pub ch: char,
    pub color: [f32; 4],
    pub background_color: Option<[f32; 4]>,
}

#[derive(Debug, Clone)]
pub struct ColoredLine {
    pub chars: Vec<ColoredChar>,
}

impl ColoredLine {
    pub fn new() -> Self {
        Self { chars: Vec::new() }
    }

    pub fn push(&mut self, ch: char, color: [f32; 4]) {
        self.chars.push(ColoredChar {
            ch,
            color,
            background_color: None,
        });
    }

    pub fn push_with_background(&mut self, ch: char, color: [f32; 4], background_color: [f32; 4]) {
        self.chars.push(ColoredChar {
            ch,
            color,
            background_color: Some(background_color),
        });
    }

    pub fn push_str(&mut self, text: &str, color: [f32; 4]) {
        for ch in text.chars() {
            self.push(ch, color);
        }
    }
}

pub enum WriteResult {
    Written,
    Overflow { writed: usize },
}

pub trait TextSurface {
    fn write_line(&mut self, line: &ColoredLine) -> WriteResult;
    fn write_line_wordwrap(&mut self, line: &ColoredLine) -> WriteResult;
    fn write_char(&mut self, ch: &ColoredChar) -> WriteResult;
    fn write_break(&mut self) -> WriteResult;
}

#[derive(Debug, Clone)]
pub struct ColoredText {
    pub lines: Vec<ColoredLine>,
}

impl ColoredText {
    pub fn new() -> Self {
        Self {
            lines: vec![ColoredLine::new()],
        }
    }

    pub fn from_str_with_color(text: &str, color: [f32; 4]) -> Self {
        let mut colored_text = Self::new();
        for ch in text.chars() {
            if ch == '\n' {
                colored_text.lines.push(ColoredLine::new());
            } else {
                if let Some(last_line) = colored_text.lines.last_mut() {
                    last_line.push(ch, color);
                }
            }
        }
        colored_text
    }

    pub fn push(&mut self, ch: char, color: [f32; 4]) {
        if ch == '\n' {
            self.lines.push(ColoredLine::new());
        } else {
            if let Some(last_line) = self.lines.last_mut() {
                last_line.push(ch, color);
            }
        }
    }

    pub fn push_with_background(&mut self, ch: char, color: [f32; 4], background_color: [f32; 4]) {
        if ch == '\n' {
            self.lines.push(ColoredLine::new());
        } else {
            if let Some(last_line) = self.lines.last_mut() {
                last_line.push_with_background(ch, color, background_color);
            }
        }
    }

    pub fn push_str(&mut self, text: &str, color: [f32; 4]) {
        for ch in text.chars() {
            self.push(ch, color);
        }
    }

    pub fn iter_chars(&self) -> impl Iterator<Item = &ColoredChar> {
        self.lines.iter().flat_map(|line| line.chars.iter())
    }

    /// Returns the total number of characters across all lines
    pub fn total_char_count(&self) -> usize {
        self.lines.iter().map(|line| line.chars.len()).sum()
    }

    /// Gets a mutable reference to a character by absolute index (across all lines)
    pub fn get_char_mut(&mut self, mut index: usize) -> Option<&mut ColoredChar> {
        for line in &mut self.lines {
            for i in 0..line.chars.len() {
                if index == 0 {
                    return line.chars.get_mut(i);
                }
                if line.chars[i].ch.len_utf8() > index {
                    return None;
                }
                index -= line.chars[i].ch.len_utf8();
            }
            if index == 0 {
                return None;
            }
            index -= '\n'.len_utf8();
        }
        None
    }

    /// Gets a reference to a character by absolute index (across all lines)
    pub fn get_char(&self, mut index: usize) -> Option<&ColoredChar> {
        for line in &self.lines {
            if index < line.chars.len() {
                return line.chars.get(index);
            }
            index -= line.chars.len();
        }
        None
    }

    /// Pushes a colored char at the end (respecting line boundaries)
    pub fn push_colored_char(&mut self, colored_char: ColoredChar) {
        if colored_char.ch == '\n' {
            self.lines.push(ColoredLine::new());
        } else {
            if let Some(last_line) = self.lines.last_mut() {
                last_line.chars.push(colored_char);
            }
        }
    }
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
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}

pub struct TextSystem {
    font: FontArc,
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    vertex_buffer: Option<Subbuffer<[TextVertex]>>,
    pub is_pipeline_ready: bool,

    // Texture atlas
    atlas_texture: Option<Arc<ImageView>>,
    atlas_sampler: Option<Arc<Sampler>>,
    glyph_infos: HashMap<char, GlyphInfo>,
    descriptor_set: Option<Arc<DescriptorSet>>,
    current_settings: TextRenderSettings,
    window_size: [f32; 2],
    vertices: Vec<TextVertex>,
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
            vertex_buffer: None,
            is_pipeline_ready: false, // Will be ready after atlas creation

            atlas_texture: None,
            atlas_sampler: None,
            glyph_infos: HashMap::new(),
            descriptor_set: None,
            current_settings: settings,
            window_size: [800.0, 600.0],
            vertices: Vec::new(),
        })
    }

    pub fn update_text_with_settings(&mut self, colored_text: &ColoredText) -> Result<()> {
        let total_chars: usize = colored_text.lines.iter().map(|line| line.chars.len()).sum();
        let mut vertices = Vec::with_capacity(total_chars * 6);
        let scale = PxScale::from(self.current_settings.font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut cursor_x = self.current_settings.position[0];
        let mut cursor_y = self.current_settings.position[1];
        let line_height = scaled_font.height();

        for line in &colored_text.lines {
            for colored_char in &line.chars {
                let ch = colored_char.ch;
                let char_color = colored_char.color;

                if ch == '\r' {
                    continue;
                }

                // Calculate advance width for this character (needed for both glyph and background)
                let advance_width = if let Some(glyph_info) = self.glyph_infos.get(&ch) {
                    glyph_info.advance
                } else {
                    let glyph_id = self.font.glyph_id(ch);
                    scaled_font.h_advance(glyph_id)
                };

                // Get glyph info from atlas
                if let Some(glyph_info) = self.glyph_infos.get(&ch) {
                    let pos_x = cursor_x + glyph_info.bearing[0];
                    let pos_y = cursor_y + glyph_info.bearing[1];

                    // Create quad vertices for this glyph using atlas UV coordinates
                    let glyph_vertices = [
                        TextVertex {
                            position: [pos_x, pos_y],
                            tex_coords: [glyph_info.uv_min[0], glyph_info.uv_min[1]],
                            color: char_color,
                        },
                        TextVertex {
                            position: [pos_x + glyph_info.size[0], pos_y],
                            tex_coords: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                            color: char_color,
                        },
                        TextVertex {
                            position: [pos_x, pos_y + glyph_info.size[1]],
                            tex_coords: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                            color: char_color,
                        },
                        TextVertex {
                            position: [pos_x + glyph_info.size[0], pos_y],
                            tex_coords: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                            color: char_color,
                        },
                        TextVertex {
                            position: [pos_x + glyph_info.size[0], pos_y + glyph_info.size[1]],
                            tex_coords: [glyph_info.uv_max[0], glyph_info.uv_max[1]],
                            color: char_color,
                        },
                        TextVertex {
                            position: [pos_x, pos_y + glyph_info.size[1]],
                            tex_coords: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                            color: char_color,
                        },
                    ];

                    vertices.extend_from_slice(&glyph_vertices);
                }

                // Render caret overlay AFTER the character (or for whitespace without glyphs)
                if let Some(bg_color) = colored_char.background_color {
                    // Create a caret overlay quad that covers the character advance
                    // The background should start at the top of the line, not at cursor_y
                    let bg_x = cursor_x;
                    let bg_y = cursor_y - scaled_font.ascent();
                    let bg_width = advance_width;
                    let bg_height = line_height;

                    // Use solid color rendering (UV [0,0] is handled specially in shader)
                    let bg_uv = [0.0, 0.0];

                    let bg_vertices = [
                        TextVertex {
                            position: [bg_x, bg_y],
                            tex_coords: bg_uv,
                            color: bg_color,
                        },
                        TextVertex {
                            position: [bg_x + bg_width, bg_y],
                            tex_coords: bg_uv,
                            color: bg_color,
                        },
                        TextVertex {
                            position: [bg_x, bg_y + bg_height],
                            tex_coords: bg_uv,
                            color: bg_color,
                        },
                        TextVertex {
                            position: [bg_x + bg_width, bg_y],
                            tex_coords: bg_uv,
                            color: bg_color,
                        },
                        TextVertex {
                            position: [bg_x + bg_width, bg_y + bg_height],
                            tex_coords: bg_uv,
                            color: bg_color,
                        },
                        TextVertex {
                            position: [bg_x, bg_y + bg_height],
                            tex_coords: bg_uv,
                            color: bg_color,
                        },
                    ];

                    vertices.extend_from_slice(&bg_vertices);
                }

                cursor_x += advance_width;
            }

            // Move to next line
            cursor_x = 10.0;
            cursor_y += line_height;
        }

        self.update_vertex_buffer(vertices)?;
        Ok(())
    }

    // Helper method for backward compatibility with &str
    pub fn update_text_with_settings_str(&mut self, text: &str) -> Result<()> {
        let colored_text = ColoredText::from_str_with_color(text, self.current_settings.color);
        self.update_text_with_settings(&colored_text)
    }

    fn update_vertex_buffer(&mut self, vertices: Vec<TextVertex>) -> Result<()> {
        if vertices.is_empty() {
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
            vertices,
        )?;

        self.vertex_buffer = Some(vertex_buffer);
        Ok(())
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

        // Save atlas as bitmap for debugging (before moving atlas_data)
        if self.should_save_debug_atlas() {
            if let Err(e) = self.save_atlas_debug_bitmap(&atlas_data, ATLAS_SIZE) {
                log::warn!("Failed to save atlas debug bitmap: {}", e);
            } else {
                log::info!("Atlas debug bitmap saved successfully");
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

    /// Check if atlas debug saving is enabled via environment variable
    fn should_save_debug_atlas(&self) -> bool {
        env::var("CARGOTAP_DEBUG_ATLAS")
            .map(|val| val.to_lowercase() == "true" || val == "1")
            .unwrap_or(false) // Default to true for now, can be changed to false later
    }

    /// Save the atlas as a bitmap image for debugging purposes
    fn save_atlas_debug_bitmap(&self, atlas_data: &[u8], atlas_size: u32) -> Result<()> {
        // Create an image buffer from the atlas data
        let img_buffer =
            ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(atlas_size, atlas_size, atlas_data.to_vec())
                .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from atlas data"))?;

        // Create filename with timestamp for uniqueness
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let filename = format!(
            "atlas_debug_{}x{}_size{}_{}.png",
            atlas_size, atlas_size, self.current_settings.font_size, timestamp
        );

        img_buffer
            .save(&filename)
            .map_err(|e| anyhow::anyhow!("Failed to save atlas image: {}", e))?;

        log::info!(
            "Atlas debug bitmap saved as: {} (utilization: {:.1}%)",
            filename,
            self.calculate_atlas_utilization(atlas_size)
        );
        Ok(())
    }

    /// Calculate atlas utilization percentage for debugging
    fn calculate_atlas_utilization(&self, _atlas_size: u32) -> f32 {
        if self.glyph_infos.is_empty() {
            return 0.0;
        }

        let mut max_y = 0.0_f32;
        for glyph_info in self.glyph_infos.values() {
            max_y = max_y.max(glyph_info.uv_max[1]);
        }

        (max_y * 100.0).min(100.0)
    }

    pub fn create_text_pipeline(&mut self) -> Result<()> {
        // This is now handled by create_text_atlas
        log::info!("Text rendering pipeline marked as ready");
        Ok(())
    }

    /// Manually save the current atlas as a debug bitmap
    /// This can be called at any time after the atlas has been created
    pub fn save_atlas_debug(&self) -> Result<()> {
        if self.atlas_texture.is_none() {
            return Err(anyhow::anyhow!("Atlas has not been created yet"));
        }

        // We need to recreate the atlas data since it's already been moved to GPU
        // This is a limitation - we can't easily read back from GPU memory
        log::warn!("Manual atlas debug save requires recreating atlas data");

        // For now, just log that this feature would need GPU readback
        log::info!(
            "To manually save atlas debug bitmap, enable CARGOTAP_DEBUG_ATLAS=true when creating the atlas"
        );

        Ok(())
    }

    pub fn draw(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        text_pipeline: Arc<GraphicsPipeline>,
        text_pipeline_layout: Arc<PipelineLayout>,
    ) -> Result<()> {
        if let (Some(vertex_buffer), Some(descriptor_set)) =
            (&self.vertex_buffer, &self.descriptor_set)
        {
            log::debug!(
                "TextSystem::draw() called with {} vertices",
                vertex_buffer.len()
            );

            if !vertex_buffer.len() > 0 {
                // Set push constants
                let push_constants = TextPushConstants {
                    screen_size: self.window_size,
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
                        .draw(vertex_buffer.len() as u32, 1, 0, 0)
                        .map_err(|e| anyhow::anyhow!("Failed to draw vertices: {}", e))?;
                }

                log::debug!(
                    "Successfully drew {} text vertices to screen with text pipeline",
                    vertex_buffer.len()
                );
            }
        } else {
            log::debug!("Vertex buffer or descriptor set not available for text rendering");
        }
        Ok(())
    }

    pub fn has_text(&self) -> bool {
        self.vertex_buffer.is_some() && self.descriptor_set.is_some()
    }

    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_size = [width, height];
    }

    pub fn get_window_size(&self) -> [f32; 2] {
        self.window_size
    }

    fn calculate_line_width(&self, line: &ColoredLine) -> f32 {
        let scale = PxScale::from(self.current_settings.font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut width = 0.0;
        for colored_char in &line.chars {
            let ch = colored_char.ch;
            if ch == '\r' {
                continue;
            }

            let advance_width = if let Some(glyph_info) = self.glyph_infos.get(&ch) {
                glyph_info.advance
            } else {
                let glyph_id = self.font.glyph_id(ch);
                scaled_font.h_advance(glyph_id)
            };

            width += advance_width;
        }

        width
    }

    fn get_line_height(&self) -> f32 {
        let scale = PxScale::from(self.current_settings.font_size);
        let scaled_font = self.font.as_scaled(scale);
        scaled_font.height()
    }

    fn get_current_cursor_y(&self) -> f32 {
        self.current_settings.position[1]
    }

    fn add_char_vertices(&mut self, colored_char: &ColoredChar) {
        let scale = PxScale::from(self.current_settings.font_size);
        let scaled_font = self.font.as_scaled(scale);
        let cursor_x = self.current_settings.position[0];
        let cursor_y = self.current_settings.position[1];
        let line_height = scaled_font.height();
        let ch = colored_char.ch;
        let char_color = colored_char.color;

        if ch == '\r' {
            return;
        }

        let advance_width = if let Some(glyph_info) = self.glyph_infos.get(&ch) {
            glyph_info.advance
        } else {
            let glyph_id = self.font.glyph_id(ch);
            scaled_font.h_advance(glyph_id)
        };

        if let Some(glyph_info) = self.glyph_infos.get(&ch) {
            let pos_x = cursor_x + glyph_info.bearing[0];
            let pos_y = cursor_y + glyph_info.bearing[1];

            let glyph_vertices = [
                TextVertex {
                    position: [pos_x, pos_y],
                    tex_coords: [glyph_info.uv_min[0], glyph_info.uv_min[1]],
                    color: char_color,
                },
                TextVertex {
                    position: [pos_x + glyph_info.size[0], pos_y],
                    tex_coords: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                    color: char_color,
                },
                TextVertex {
                    position: [pos_x, pos_y + glyph_info.size[1]],
                    tex_coords: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                    color: char_color,
                },
                TextVertex {
                    position: [pos_x + glyph_info.size[0], pos_y],
                    tex_coords: [glyph_info.uv_max[0], glyph_info.uv_min[1]],
                    color: char_color,
                },
                TextVertex {
                    position: [pos_x + glyph_info.size[0], pos_y + glyph_info.size[1]],
                    tex_coords: [glyph_info.uv_max[0], glyph_info.uv_max[1]],
                    color: char_color,
                },
                TextVertex {
                    position: [pos_x, pos_y + glyph_info.size[1]],
                    tex_coords: [glyph_info.uv_min[0], glyph_info.uv_max[1]],
                    color: char_color,
                },
            ];

            self.vertices.extend_from_slice(&glyph_vertices);
        }

        if let Some(bg_color) = colored_char.background_color {
            let bg_x = cursor_x;
            let bg_y = cursor_y - scaled_font.ascent();
            let bg_width = advance_width;
            let bg_height = line_height;
            let bg_uv = [0.0, 0.0];

            let bg_vertices = [
                TextVertex {
                    position: [bg_x, bg_y],
                    tex_coords: bg_uv,
                    color: bg_color,
                },
                TextVertex {
                    position: [bg_x + bg_width, bg_y],
                    tex_coords: bg_uv,
                    color: bg_color,
                },
                TextVertex {
                    position: [bg_x, bg_y + bg_height],
                    tex_coords: bg_uv,
                    color: bg_color,
                },
                TextVertex {
                    position: [bg_x + bg_width, bg_y],
                    tex_coords: bg_uv,
                    color: bg_color,
                },
                TextVertex {
                    position: [bg_x + bg_width, bg_y + bg_height],
                    tex_coords: bg_uv,
                    color: bg_color,
                },
                TextVertex {
                    position: [bg_x, bg_y + bg_height],
                    tex_coords: bg_uv,
                    color: bg_color,
                },
            ];

            self.vertices.extend_from_slice(&bg_vertices);
        }
    }

    pub fn flush_vertices(&mut self) -> Result<()> {
        if self.vertices.is_empty() {
            return Ok(());
        }
        let vertices = std::mem::take(&mut self.vertices);
        self.update_vertex_buffer(vertices)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.current_settings.position = [10.0, 30.0];
    }
}

impl TextSurface for TextSystem {
    fn write_line(&mut self, line: &ColoredLine) -> WriteResult {
        let mut total_writed = 0usize;
        for ch in line.chars.iter() {
            match self.write_char(&ch) {
                WriteResult::Written => {
                    total_writed += 1;
                }
                WriteResult::Overflow { writed } => {
                    return WriteResult::Overflow {
                        writed: total_writed + writed,
                    };
                }
            };
        }

        return WriteResult::Written;
    }

    fn write_line_wordwrap(&mut self, line: &ColoredLine) -> WriteResult {
        let mut total_writed = 0usize;
        let mut line = ColoredLine {
            chars: line.chars.clone(),
        };
        loop {
            match self.write_line(&line) {
                WriteResult::Written => return WriteResult::Written,
                WriteResult::Overflow { writed } => {
                    total_writed += writed;
                    line.chars = line.chars.split_off(writed);
                    if matches!(self.write_break(), WriteResult::Overflow { writed: _ }) {
                        return WriteResult::Overflow {
                            writed: total_writed,
                        };
                    }
                }
            }
        }
    }

    fn write_char(&mut self, ch: &ColoredChar) -> WriteResult {
        if ch.ch == '\n' {
            return self.write_break();
        }

        if ch.ch == '\r' {
            return WriteResult::Written;
        }

        let scale = PxScale::from(self.current_settings.font_size);

        let advance_width = if let Some(glyph_info) = self.glyph_infos.get(&ch.ch) {
            glyph_info.advance
        } else {
            let glyph_id = self.font.glyph_id(ch.ch);
            let scaled_font = self.font.as_scaled(scale);
            scaled_font.h_advance(glyph_id)
        };

        let cursor_x = self.current_settings.position[0];
        let cursor_y = self.current_settings.position[1];
        let line_height = self.get_line_height();
        let ascent = {
            let scaled_font = self.font.as_scaled(scale);
            scaled_font.ascent()
        };
        let total_line_height = cursor_y + line_height - ascent;

        if cursor_x + advance_width > self.window_size[0] || total_line_height > self.window_size[1]
        {
            return WriteResult::Overflow { writed: 0 };
        }

        self.add_char_vertices(ch);
        self.current_settings.position[0] += advance_width;
        WriteResult::Written
    }

    fn write_break(&mut self) -> WriteResult {
        let line_height = self.get_line_height();
        self.current_settings.position[0] = 10.0;
        self.current_settings.position[1] += line_height;

        let scale = PxScale::from(self.current_settings.font_size);
        let ascent = {
            let scaled_font = self.font.as_scaled(scale);
            scaled_font.ascent()
        };
        let total_height = self.current_settings.position[1] + line_height - ascent;

        if total_height > self.window_size[1] {
            return WriteResult::Overflow { writed: 0 };
        }

        return WriteResult::Written;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_line_basic() {
        let mut line = ColoredLine::new();
        assert_eq!(line.chars.len(), 0);

        line.push('H', [1.0, 1.0, 1.0, 1.0]);
        line.push('i', [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(line.chars.len(), 2);
        assert_eq!(line.chars[0].ch, 'H');
        assert_eq!(line.chars[1].ch, 'i');
    }

    #[test]
    fn test_colored_line_with_background() {
        let mut line = ColoredLine::new();
        line.push_with_background('A', [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0]);

        assert_eq!(line.chars.len(), 1);
        assert_eq!(line.chars[0].ch, 'A');
        assert!(line.chars[0].background_color.is_some());
        assert_eq!(
            line.chars[0].background_color.unwrap(),
            [0.0, 0.0, 1.0, 1.0]
        );
    }

    #[test]
    fn test_colored_text_basic() {
        let mut text = ColoredText::new();
        assert_eq!(text.lines.len(), 1);

        text.push('H', [1.0, 1.0, 1.0, 1.0]);
        text.push('i', [1.0, 1.0, 1.0, 1.0]);

        assert_eq!(text.lines.len(), 1);
        assert_eq!(text.lines[0].chars.len(), 2);
    }

    #[test]
    fn test_colored_text_newline() {
        let mut text = ColoredText::new();
        text.push('H', [1.0, 1.0, 1.0, 1.0]);
        text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        text.push('i', [1.0, 1.0, 1.0, 1.0]);

        assert_eq!(text.lines.len(), 2);
        assert_eq!(text.lines[0].chars.len(), 1);
        assert_eq!(text.lines[1].chars.len(), 1);
    }

    #[test]
    fn test_colored_text_from_str() {
        let text = ColoredText::from_str_with_color("Hello\nWorld", [1.0, 1.0, 1.0, 1.0]);

        assert_eq!(text.lines.len(), 2);
        assert_eq!(text.lines[0].chars.len(), 5);
        assert_eq!(text.lines[1].chars.len(), 5);
    }

    #[test]
    fn test_colored_text_total_char_count() {
        let mut text = ColoredText::new();
        text.push_str("Hello", [1.0, 1.0, 1.0, 1.0]);
        text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        text.push_str("World", [1.0, 1.0, 1.0, 1.0]);

        assert_eq!(text.total_char_count(), 10);
    }

    #[test]
    fn test_write_result_overflow() {
        let result = WriteResult::Overflow { writed: 5 };
        if let WriteResult::Overflow { writed } = result {
            assert_eq!(writed, 5);
        } else {
            panic!("Expected Overflow variant");
        }
    }
}
