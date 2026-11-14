use crate::colour::Colour;
use eframe::{
    egui::{PaintCallbackInfo, Response, Sense, Ui, Widget},
    egui_wgpu::{Callback, CallbackResources, CallbackTrait, ScreenDescriptor},
    wgpu::{
        AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
        BufferBinding, BufferBindingType, BufferUsages, Color, CommandBuffer, CommandEncoder,
        Device, Extent3d, FilterMode, FragmentState, LoadOp, MipmapFilterMode, MultisampleState,
        Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue,
        RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
        RenderPipelineDescriptor, Sampler, SamplerBindingType, ShaderModule,
        ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TexelCopyBufferLayout,
        Texture, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
        TextureViewDimension, VertexState,
        wgt::{BufferDescriptor, SamplerDescriptor, TextureDescriptor, TextureViewDescriptor},
    },
};
use std::{num::NonZeroU64, sync::Arc, time::Instant};

const BLUR_SIZE: u32 = 128;
const BLUR_PASSES: u32 = 40;

type Pad<const N: usize> = [f32; N];

#[derive(Debug, Clone, Copy)]
struct Options {
    pub warp_intensity: f32,
    pub blur_passes: u32,
    pub animation_speed: f32,
    pub transition_duration_ms: f32,
    pub saturation: f32,
    pub tint_colour: [f32; 3],
    pub tint_intensity: f32,
    pub dithering: f32,
    pub scale: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            warp_intensity: 1.0,
            blur_passes: 8,
            animation_speed: 1.0,
            transition_duration_ms: 1000.0,
            saturation: 1.5,
            tint_colour: [0.16, 0.16, 0.24],
            tint_intensity: 0.15,
            dithering: 0.008,
            scale: 1.0,
        }
    }
}

struct SourceImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl SourceImage {
    fn new(colours: &[[f32; 4]], angle_deg: f32) -> Self {
        let s = 512;
        let sf = s as f32;

        let rad = angle_deg.to_radians();
        let (x1, y1) = (sf / 2.0 - rad.cos() * sf, sf / 2.0 - rad.sin() * sf);
        let (x2, y2) = (sf / 2.0 + rad.cos() * sf, sf / 2.0 + rad.sin() * sf);

        let (dx, dy) = (x2 - x1, y2 - y1);
        let len_sq = dx * dx + dy * dy;
        let n = colours.len();

        let mut rgba = Vec::with_capacity(s * s * 4);
        for py in 0..s {
            for px in 0..s {
                let t = (((px as f32 + 0.5 - x1) * dx + (py as f32 + 0.5 - y1) * dy) / len_sq)
                    .clamp(0.0, 1.0);
                let f = t * (n - 1) as f32;
                let i = (f.floor() as usize).min(n - 2);
                let k = f - i as f32;
                for c in 0..4 {
                    let v = colours[i][c] * (1.0 - k) + colours[i + 1][c] * k;
                    rgba.push((v.clamp(0.0, 1.0) * 255.0).round() as u8);
                }
            }
        }

        Self {
            rgba,
            width: s as u32,
            height: s as u32,
        }
    }
}

struct Pass {
    pipeline: RenderPipeline,
    bind_group: BindGroupLayout,
}

impl Pass {
    fn new(
        device: &Device,
        shader: &ShaderModule,
        label: &str,
        vs: &str,
        fs: &str,
        min_binding_size: u64,
        has_dynamic_offset: bool,
        textures: u32,
        format: TextureFormat,
    ) -> Self {
        let mut entries = vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset,
                    min_binding_size: NonZeroU64::new(min_binding_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ];

        for i in 0..textures {
            entries.push(BindGroupLayoutEntry {
                binding: i + 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
        }

        let bind_group = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &entries,
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &[Some(&bind_group)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            vertex: VertexState {
                module: shader,
                entry_point: Some(vs),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: shader,
                entry_point: Some(fs),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
        }
    }
}

struct Target {
    texture: Texture,
    view: TextureView,
    size: [u32; 2],
}

impl Target {
    fn new(device: &Device, label: &str, w: u32, h: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view,
            size: [w, h],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurUniforms {
    resolution: [f32; 2],
    offset: f32,
    __: Pad<1>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlendUniforms {
    blend: f32,
    __: Pad<3>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TintUniforms {
    tint_colour: [f32; 3],
    tint_intensity: f32,
    __: Pad<0>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WarpUniforms {
    time: f32,
    intensity: f32,
    __: Pad<2>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OutputUniforms {
    saturation: f32,
    dithering: f32,
    time: f32,
    scale: f32,
    resolution: [f32; 2],
    __: Pad<2>,
}

fn bind(
    device: &Device,
    pass: &Pass,
    buffer: &Buffer,
    min_binding_size: u64,
    sampler: &Sampler,
    views: &[&TextureView],
) -> BindGroup {
    let mut entries = vec![
        BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer,
                offset: 0,
                size: NonZeroU64::new(min_binding_size),
            }),
        },
        BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(sampler),
        },
    ];

    for (i, v) in views.iter().enumerate() {
        entries.push(BindGroupEntry {
            binding: i as u32 + 2,
            resource: BindingResource::TextureView(v),
        });
    }

    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pass.bind_group,
        entries: &entries,
    })
}

fn render_pass(
    encoder: &mut CommandEncoder,
    label: &str,
    target: &TextureView,
    pass: &Pass,
    bind_group: &BindGroup,
    dynamic_offsets: &[u32],
) {
    let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    rp.set_pipeline(&pass.pipeline);
    rp.set_bind_group(0, bind_group, dynamic_offsets);
    rp.draw(0..6, 0..1);
}

struct DynamicBgRenderResources {
    sampler: Sampler,

    tint: Pass,
    blur: Pass,
    blend: Pass,
    warp: Pass,
    output: Pass,

    tint_buffer: Buffer,
    blur_buffer: Buffer,
    blur_stride: u64,
    blend_buffer: Buffer,
    warp_buffer: Buffer,
    output_buffer: Buffer,

    blur_a: Target,
    blur_b: Target,
    albums: [Target; 2],
    blend_out: Target,
    warp_out: Target,
    source: Option<Texture>,

    tint_bind_group: Option<BindGroup>,
    blur_bind_group: [BindGroup; 2],
    blend_bind_group: [BindGroup; 2],
    warp_bind_group: [BindGroup; 3],
    output_bind_group: BindGroup,

    generation: u64,
    has_image: bool,
    current: usize,
    transition_start: Option<f32>,
    last_time: f32,
    time: f32,
}

impl DynamicBgRenderResources {
    fn new(device: &Device, shader: &ShaderModule, format: TextureFormat) -> Self {
        let buffer = |label: &str, size: u64| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("dynamic_bg"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let tint = Pass::new(
            device,
            shader,
            "dynamic_bg_tint",
            "vs_offscreen",
            "fs_tint",
            16,
            false,
            1,
            TextureFormat::Rgba16Float,
        );

        let blur = Pass::new(
            device,
            shader,
            "dynamic_bg_blur",
            "vs_offscreen",
            "fs_blur",
            16,
            true,
            1,
            TextureFormat::Rgba16Float,
        );

        let blend = Pass::new(
            device,
            shader,
            "dynamic_bg_blend",
            "vs_offscreen",
            "fs_blend",
            16,
            false,
            2,
            TextureFormat::Rgba16Float,
        );

        let warp = Pass::new(
            device,
            shader,
            "dynamic_bg_warp",
            "vs_offscreen",
            "fs_warp",
            16,
            false,
            1,
            TextureFormat::Rgba16Float,
        );

        let output = Pass::new(
            device,
            shader,
            "dynamic_bg_output",
            "vs_main",
            "fs_output",
            32,
            false,
            1,
            format,
        );

        let tint_buffer = buffer("dynamic_bg_tint", 16);
        let blur_stride = (device.limits().min_uniform_buffer_offset_alignment as u64).max(16);
        let blur_buffer = buffer("dynamic_bg_blur", blur_stride * BLUR_PASSES as u64);
        let blend_buffer = buffer("dynamic_bg_blend", 16);
        let warp_buffer = buffer("dynamic_bg_warp", 16);
        let output_buffer = buffer("dynamic_bg_output", 32);

        let blur_a = Target::new(device, "dynamic_bg_blur_a", BLUR_SIZE, BLUR_SIZE);
        let blur_b = Target::new(device, "dynamic_bg_blur_b", BLUR_SIZE, BLUR_SIZE);
        let albums = [
            Target::new(device, "dynamic_bg_album_0", BLUR_SIZE, BLUR_SIZE),
            Target::new(device, "dynamic_bg_album_1", BLUR_SIZE, BLUR_SIZE),
        ];

        let blend_out = Target::new(device, "dynamic_bg_blend_out", BLUR_SIZE, BLUR_SIZE);
        let warp_out = Target::new(device, "dynamic_bg_warp_out", 640, 360);

        let blur_bind_group = [
            bind(device, &blur, &blur_buffer, 16, &sampler, &[&blur_a.view]),
            bind(device, &blur, &blur_buffer, 16, &sampler, &[&blur_b.view]),
        ];

        let blend_bind_group = [
            bind(
                device,
                &blend,
                &blend_buffer,
                16,
                &sampler,
                &[&albums[0].view, &albums[1].view],
            ),
            bind(
                device,
                &blend,
                &blend_buffer,
                16,
                &sampler,
                &[&albums[1].view, &albums[0].view],
            ),
        ];

        let warp_bind_group = [
            bind(
                device,
                &warp,
                &warp_buffer,
                16,
                &sampler,
                &[&albums[0].view],
            ),
            bind(
                device,
                &warp,
                &warp_buffer,
                16,
                &sampler,
                &[&albums[1].view],
            ),
            bind(
                device,
                &warp,
                &warp_buffer,
                16,
                &sampler,
                &[&blend_out.view],
            ),
        ];

        let output_bind_group = bind(
            device,
            &output,
            &output_buffer,
            32,
            &sampler,
            &[&warp_out.view],
        );

        Self {
            sampler,
            tint,
            blur,
            blend,
            warp,
            output,
            tint_buffer,
            blur_buffer,
            blur_stride,
            blend_buffer,
            warp_buffer,
            output_buffer,
            blur_a,
            blur_b,
            albums,
            blend_out,
            warp_out,
            source: None,
            tint_bind_group: None,
            blur_bind_group,
            blend_bind_group,
            warp_bind_group,
            output_bind_group,
            generation: 0,
            has_image: false,
            current: 0,
            transition_start: None,
            last_time: 0.0,
            time: 0.0,
        }
    }

    fn set_source(&mut self, device: &Device, queue: &Queue, img: &SourceImage) {
        let size = Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("dynamic_bg_source"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &img.rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * img.width),
                rows_per_image: Some(img.height),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        self.source = Some(texture);
        self.tint_bind_group = Some(bind(
            device,
            &self.tint,
            &self.tint_buffer,
            16,
            &self.sampler,
            &[&view],
        ));
    }

    fn process(
        &mut self,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        time: f32,
        options: &Options,
    ) {
        if self.transition_start.take().is_some() {
            self.current = 1 - self.current;
        }

        queue.write_buffer(
            &self.tint_buffer,
            0,
            bytemuck::bytes_of(&TintUniforms {
                tint_colour: options.tint_colour,
                tint_intensity: options.tint_intensity,
                __: [],
            }),
        );

        let passes = (options.blur_passes).clamp(1, BLUR_PASSES) as usize;
        for i in 0..passes {
            queue.write_buffer(
                &self.blur_buffer,
                i as u64 * self.blur_stride,
                bytemuck::bytes_of(&BlurUniforms {
                    resolution: [BLUR_SIZE as f32; 2],
                    offset: i as f32 + 0.5,
                    __: [0.0],
                }),
            );
        }

        render_pass(
            encoder,
            "dynamic_bg_tint",
            &self.blur_a.view,
            &self.tint,
            &self.tint_bind_group.as_ref().unwrap(),
            &[],
        );

        for i in 0..passes {
            let offset = (i as u64 * self.blur_stride) as u32;
            let (bind_group, dest) = if i % 2 == 0 {
                (&self.blur_bind_group[0], &self.blur_b)
            } else {
                (&self.blur_bind_group[1], &self.blur_a)
            };

            render_pass(
                encoder,
                "dynamic_bg_blur",
                &dest.view,
                &self.blur,
                bind_group,
                &[offset],
            );
        }

        let result = if passes % 2 == 0 {
            &self.blur_a
        } else {
            &self.blur_b
        };

        let dest_idx = if self.has_image {
            1 - self.current
        } else {
            self.current
        };

        encoder.copy_texture_to_texture(
            result.texture.as_image_copy(),
            self.albums[dest_idx].texture.as_image_copy(),
            Extent3d {
                width: BLUR_SIZE,
                height: BLUR_SIZE,
                depth_or_array_layers: 1,
            },
        );

        if self.has_image {
            self.transition_start = Some(time);
        } else {
            self.has_image = true;
        }
    }

    fn update(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        callback: &DynamicBgCallback,
    ) {
        let options = &callback.options;
        let dt = (callback.time - self.last_time).max(0.0);
        self.last_time = callback.time;
        self.time += dt * options.animation_speed;

        if self.warp_out.size != callback.size_px {
            self.warp_out = Target::new(
                device,
                "dynamic_bg_warp_out",
                callback.size_px[0],
                callback.size_px[1],
            );

            self.output_bind_group = bind(
                device,
                &self.output,
                &self.output_buffer,
                32,
                &self.sampler,
                &[&self.warp_out.view],
            );
        }

        if callback.generation != self.generation {
            if let Some(img) = &callback.image {
                self.set_source(device, queue, img);
                self.process(queue, encoder, callback.time, options);
            }

            self.generation = callback.generation;
        }

        if !self.has_image {
            return;
        }

        let mut blending = None;
        if let Some(start) = self.transition_start {
            let blend = if options.transition_duration_ms <= 0.0 {
                1.0
            } else {
                ((callback.time - start) * 1000.0 / options.transition_duration_ms).clamp(0.0, 1.0)
            };

            if blend >= 1.0 {
                self.current = 1 - self.current;
                self.transition_start = None;
            } else {
                blending = Some(blend);
            }
        }

        if let Some(blend) = blending {
            queue.write_buffer(
                &self.blend_buffer,
                0,
                bytemuck::bytes_of(&BlendUniforms {
                    blend,
                    __: [0.0; 3],
                }),
            );

            render_pass(
                encoder,
                "dynamic_bg_blend",
                &self.blend_out.view,
                &self.blend,
                &self.blend_bind_group[self.current],
                &[],
            );
        }

        queue.write_buffer(
            &self.warp_buffer,
            0,
            bytemuck::bytes_of(&WarpUniforms {
                time: self.time,
                intensity: options.warp_intensity,
                __: [0.0; 2],
            }),
        );

        let warp_idx = if blending.is_some() { 2 } else { self.current };
        render_pass(
            encoder,
            "dynamic_bg_warp",
            &self.warp_out.view,
            &self.warp,
            &self.warp_bind_group[warp_idx],
            &[],
        );

        queue.write_buffer(
            &self.output_buffer,
            0,
            bytemuck::bytes_of(&OutputUniforms {
                saturation: options.saturation,
                dithering: options.dithering,
                time: self.time,
                scale: options.scale,
                resolution: [callback.size_px[0] as f32, callback.size_px[1] as f32],
                __: [0.0; 2],
            }),
        );
    }

    fn paint(&self, render_pass: &mut RenderPass<'_>) {
        if !self.has_image {
            return;
        }

        render_pass.set_pipeline(&self.output.pipeline);
        render_pass.set_bind_group(0, &self.output_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

struct DynamicBgCallback {
    time: f32,
    size_px: [u32; 2],
    options: Options,
    image: Option<Arc<SourceImage>>,
    generation: u64,
}

impl CallbackTrait for DynamicBgCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        egui_encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        resources
            .get_mut::<DynamicBgRenderResources>()
            .unwrap()
            .update(device, queue, egui_encoder, self);

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
    options: Options,
    image: Option<Arc<SourceImage>>,
    generation: u64,
    last_colours: Vec<[f32; 4]>,
    first_update: Instant,
}

impl DynamicBg {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("dynamicbg"),
            source: ShaderSource::Wgsl(include_str!("../../assets/dynamicbg.wgsl").into()),
        });

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(DynamicBgRenderResources::new(
                device,
                &shader,
                wgpu_render_state.target_format,
            ));

        Some(Self {
            options: Options::default(),
            image: None,
            generation: 0,
            last_colours: Vec::new(),
            first_update: Instant::now(),
        })
    }

    // pub fn set_image(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
    //     self.generation += 1;
    //     self.image = Some(Arc::new(SourceImage {
    //         rgba,
    //         width,
    //         height,
    //     }));
    // }

    pub fn set_colours(&mut self, colours: Vec<Colour>) {
        if colours.len() < 2 {
            return;
        }

        let colours = colours
            .into_iter()
            .map(Colour::into_array_f32)
            .collect::<Vec<_>>();

        let img = SourceImage::new(&colours, 135.0);
        self.last_colours = colours;
        self.generation += 1;
        self.image = Some(Arc::new(img));
    }
}

impl Widget for &mut DynamicBg {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::empty());
        let ppp = ui.ctx().pixels_per_point();
        let size_px = [
            ((rect.width() * ppp).round() as u32).max(1),
            ((rect.height() * ppp).round() as u32).max(1),
        ];

        ui.painter().add(Callback::new_paint_callback(
            rect,
            DynamicBgCallback {
                time: self.first_update.elapsed().as_secs_f32(),
                size_px,
                options: self.options,
                image: self.image.clone(),
                generation: self.generation,
            },
        ));

        ui.request_repaint(); // continuous widget

        response
    }
}
