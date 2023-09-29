use winit::window::Window;

pub struct Renderer {
    pub window: Window,

    //instance: wgpu::Instance,
    //adapter: wgpu::Adapter,
    device: wgpu::Device,
    surface: wgpu::Surface,
    surface_format: wgpu::TextureFormat,
    storage_format: wgpu::TextureFormat,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    color_buffer: wgpu::Texture,
    color_buffer_view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    ray_tracing_pipeline: wgpu::ComputePipeline,
    ray_tracing_bind_group: wgpu::BindGroup,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        // Create the instance, adapter, device, and queue, and setup the surface
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: Some("Device"),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let storage_format = wgpu::TextureFormat::Rgba8Unorm;

        // Create the color buffer and sampler
        let color_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Buffer"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: storage_format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[storage_format],
        });

        let color_buffer_view = color_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Color Buffer Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        // Create pipelines
        let ray_tracing_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: storage_format,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });

        let ray_tracing_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Bind Group"),
            layout: &ray_tracing_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&color_buffer_view),
            }],
        });

        let ray_tracing_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Pipeline Layout"),
            bind_group_layouts: &[&ray_tracing_bind_group_layout],
            push_constant_ranges: &[],
        });

        let ray_tracing_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracing Pipeline"),
            layout: Some(&ray_tracing_pipeline_layout),
            module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Ray Tracing Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("ray_tracing_kernel.wgsl").into()),
            }),
            entry_point: "main",
        });

        let screen_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Screen Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen Bind Group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&color_buffer_view),
            }],
        });

        let screen_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pipeline Layout"),
            bind_group_layouts: &[&screen_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Screen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("screen_shader.wgsl").into()),
        });

        let screen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Pipeline"),
            layout: Some(&screen_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vert_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "frag_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Renderer {
            window,
            //adapter,
            surface_format,
            storage_format,
            //instance,
            surface,
            device,
            queue,
            config,
            size,
            color_buffer,
            color_buffer_view,
            sampler,
            ray_tracing_bind_group,
            ray_tracing_pipeline,
            screen_bind_group,
            screen_pipeline,
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut ray_trace_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Pass"),
            });
            ray_trace_pass.set_pipeline(&self.ray_tracing_pipeline);
            ray_trace_pass.set_bind_group(0, &self.ray_tracing_bind_group, &[]);
            ray_trace_pass.dispatch_workgroups(self.size.width, self.size.height, 1);
        }

        let output = self.surface.get_current_texture()?;
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color{
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.screen_pipeline);
            render_pass.set_bind_group(0, &self.screen_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        // Create a new color buffer with the new size
        self.color_buffer = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Buffer"),
            size: wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.storage_format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[self.storage_format],
        });

        self.color_buffer_view = self.color_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        // Create pipelines

        let ray_tracing_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: self.storage_format,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });

        self.ray_tracing_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Bind Group"),
            layout: &ray_tracing_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.color_buffer_view),
            }],
        });

        let ray_tracing_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Pipeline Layout"),
            bind_group_layouts: &[&ray_tracing_bind_group_layout],
            push_constant_ranges: &[],
        });

        self.ray_tracing_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracing Pipeline"),
            layout: Some(&ray_tracing_pipeline_layout),
            module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Ray Tracing Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("ray_tracing_kernel.wgsl").into()),
            }),
            entry_point: "main",
        });

        let screen_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Screen Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        self.screen_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen Bind Group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&self.color_buffer_view),
            }],
        });

        let screen_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pipeline Layout"),
            bind_group_layouts: &[&screen_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Screen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("screen_shader.wgsl").into()),
        });

        self.screen_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Pipeline"),
            layout: Some(&screen_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vert_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "frag_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
    }
}
