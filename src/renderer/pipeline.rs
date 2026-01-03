use std::sync::Arc;
use vulkano::{
    buffer::BufferContents,
    device::Device,
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::PipelineRenderingCreateInfo,
            vertex_input::{Vertex, VertexDefinition},
            viewport::ViewportState,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    swapchain::Swapchain,
};

use crate::text::TextVertex;

// We use `#[repr(C)]` here to force rustc to use a defined layout for our data, as the default
// representation has *no guarantees*.
#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

// Vertex shader for triangle rendering
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
    }
}

// Fragment shader for triangle rendering
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
    }
}

// Vertex shader for text rendering
mod text_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 tex_coords;
            layout(location = 2) in vec4 color;

            layout(location = 0) out vec2 frag_tex_coords;
            layout(location = 1) out vec4 frag_color;

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec4 text_color;
            } pc;

            void main() {
                // Convert screen coordinates to normalized device coordinates
                vec2 normalized_pos = (position / pc.screen_size) * 2.0 - 1.0;
                // normalized_pos.y = -normalized_pos.y; // Flip Y coordinate

                gl_Position = vec4(normalized_pos, 0.0, 1.0);
                frag_tex_coords = tex_coords;
                frag_color = color;
            }
        ",
    }
}

// Fragment shader for text rendering
mod text_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec2 frag_tex_coords;
            layout(location = 1) in vec4 frag_color;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D glyph_texture;

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec4 text_color;
            } pc;

            void main() {
                float alpha = texture(glyph_texture, frag_tex_coords).r;

                // Use per-vertex color instead of push constant color
                f_color = vec4(frag_color.rgb * alpha, frag_color.a);

                // Discard fully transparent pixels
                if (alpha < 0.01) {
                    discard;
                }
            }
        ",
    }
}

/// Creates the main graphics pipeline for triangle rendering
pub fn create_graphics_pipeline(
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
) -> Arc<GraphicsPipeline> {
    // Load shaders
    let vs = vs::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let fs = fs::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();

    // Automatically generate vertex input state from the vertex shader's input interface
    let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

    // Make a list of the shader stages that the pipeline will have
    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    // Create pipeline layout
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    // Describe the formats of attachment images
    let subpass = PipelineRenderingCreateInfo {
        color_attachment_formats: vec![Some(swapchain.image_format())],
        ..Default::default()
    };

    // Create the pipeline
    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.color_attachment_formats.len() as u32,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

/// Creates the text rendering pipeline
pub fn create_text_pipeline(
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
) -> (Arc<GraphicsPipeline>, Arc<PipelineLayout>) {
    // Load text shaders
    let text_vs = text_vs::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let text_fs = text_fs::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();

    // Create vertex input state for text
    let text_vertex_input_state = TextVertex::per_vertex().definition(&text_vs).unwrap();

    let text_stages = [
        PipelineShaderStageCreateInfo::new(text_vs),
        PipelineShaderStageCreateInfo::new(text_fs),
    ];

    // Create text pipeline layout
    let text_layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&text_stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let text_subpass = PipelineRenderingCreateInfo {
        color_attachment_formats: vec![Some(swapchain.image_format())],
        ..Default::default()
    };

    // Create text pipeline
    let text_pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: text_stages.into_iter().collect(),
            vertex_input_state: Some(text_vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                text_subpass.color_attachment_formats.len() as u32,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(text_subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(text_layout.clone())
        },
    )
    .unwrap();

    (text_pipeline, text_layout)
}
