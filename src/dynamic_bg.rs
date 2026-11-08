use std::num::NonZeroU64;

use eframe::{
    egui::{PaintCallbackInfo, Response, Ui, Widget},
    egui_wgpu::{Callback, CallbackResources, CallbackTrait},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
        BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, FragmentState,
        MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState,
        RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
        ShaderStages, VertexState,
        util::{BufferInitDescriptor, DeviceExt},
    },
};

struct DynamicBgRenderResources {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
}

impl DynamicBgRenderResources {
    fn paint(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

struct DynamicBgCallback {}

impl CallbackTrait for DynamicBgCallback {
    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        resources
            .get::<DynamicBgRenderResources>()
            .unwrap()
            .paint(render_pass);
    }
}

pub struct DynamicBg {}

impl DynamicBg {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("dynamicbg"),
            source: ShaderSource::Wgsl(include_str!("../assets/dynamicbg.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("dynamicbg"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("dynamicbg"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("dynamicbg"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu_render_state.target_format.into())],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("dynamicbg"),
            contents: bytemuck::cast_slice(&[0.0_f32; 4]),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dynamicbg"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(DynamicBgRenderResources {
                pipeline,
                bind_group,
            });

        Some(Self {})
    }
}

impl Widget for &mut DynamicBg {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = ui.max_rect();

        ui.painter()
            .add(Callback::new_paint_callback(rect, DynamicBgCallback {}));
        ui.request_repaint(); // continuous widget

        ui.response()
    }
}
