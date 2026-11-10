use crate::colour::Colour;
use eframe::{
    egui::{PaintCallbackInfo, Response, Sense, Ui, Widget},
    egui_wgpu::{Callback, CallbackResources, CallbackTrait, ScreenDescriptor},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
        BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, CommandBuffer,
        CommandEncoder, Device, FragmentState, MultisampleState, PipelineCompilationOptions,
        PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
        RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexState,
        util::{BufferInitDescriptor, DeviceExt},
    },
};
use std::{num::NonZeroU64, time::Instant};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DynamicBgUniforms {
    delta: f32,
    _1: [f32; 3],
    time: f32,
    _2: [f32; 3],
    colours: [[f32; 4]; 4],
}

struct DynamicBgRenderResources {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
}

impl DynamicBgRenderResources {
    fn paint(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    pub fn write_uniforms(&self, queue: &Queue, delta: f32, time: f32, colours: [[f32; 4]; 4]) {
        let uniforms = DynamicBgUniforms {
            delta,
            _1: [0.0; 3],
            time,
            _2: [0.0; 3],
            colours,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

struct DynamicBgCallback {
    delta: f32,
    time: f32,
    colours: [[f32; 4]; 4],
}

impl CallbackTrait for DynamicBgCallback {
    fn prepare(
        &self,
        _device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        resources
            .get::<DynamicBgRenderResources>()
            .unwrap()
            .write_uniforms(queue, self.delta, self.time, self.colours);

        Vec::new()
    }

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

pub struct DynamicBg {
    colours: [[f32; 4]; 4],
    first_update: Instant,
    last_update: Instant,
}

impl DynamicBg {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("dynamicbg"),
            source: ShaderSource::Wgsl(include_str!("../../assets/dynamicbg.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("dynamicbg"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(96),
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
            contents: bytemuck::cast_slice(&[DynamicBgUniforms {
                delta: 0.0,
                _1: [0.0; 3],
                time: 0.0,
                _2: [0.0; 3],
                colours: [[0.0; 4]; 4],
            }]),
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
                uniform_buffer,
            });

        Some(Self {
            colours: [[0.0; 4]; 4],
            first_update: Instant::now(),
            last_update: Instant::now(),
        })
    }

    pub fn set_colours(&mut self, colours: Vec<Colour>) {
        if colours.len() < 4 {
            return;
        }

        let colours = *colours
            .into_iter()
            .take(4)
            .map(Colour::into_array_f32)
            .collect::<Vec<[f32; 4]>>()
            .as_array::<4>()
            .unwrap();

        self.colours = colours;
    }
}

impl Widget for &mut DynamicBg {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::empty());
        let now = Instant::now();

        ui.painter().add(Callback::new_paint_callback(
            rect,
            DynamicBgCallback {
                colours: self.colours,
                delta: (now - self.last_update).as_secs_f32(),
                time: (now - self.first_update).as_secs_f32(),
            },
        ));

        self.last_update = now;
        ui.request_repaint(); // continuous widget

        response
    }
}
