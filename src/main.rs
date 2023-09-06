struct App {
    instance : wgpu::Instance,
    surface : wgpu::Surface,
    adapter : wgpu::Adapter,
    device : wgpu::Device,
    queue : wgpu::Queue,
    surface_caps : wgpu::SurfaceCapabilities,
    surface_configuration : wgpu::SurfaceConfiguration,

    window_size : winit::dpi::PhysicalSize<u32>,

    render_pipeline : wgpu::RenderPipeline
}

impl App {
    async fn new(window : &winit::window::Window) -> Self {

        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{
            backends : wgpu::Backends::all(),
            dx12_shader_compiler : wgpu::Dx12Compiler::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions{
            power_preference : wgpu::PowerPreference::HighPerformance,
            compatible_surface : Some(&surface),
            force_fallback_adapter : false
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            features : wgpu::Features::empty(),
            limits : wgpu::Limits::default(),
            label : None
        }, None).await.unwrap();


        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format : surface_format,
            width : window_size.width,
            height : window_size.height,
            present_mode : surface_caps.present_modes[0],
            alpha_mode : surface_caps.alpha_modes[0],
            view_formats : vec![],
        };

        surface.configure(&device, &surface_configuration);

        //shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : Some("Render Pipeline Layout"),
            bind_group_layouts : &[],
            push_constant_ranges : &[]
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : Some("Render Pipeline"),
            layout : Some(&render_pipline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers : &[]
            },
            fragment : Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                //window output setting:
                targets : &[Some(wgpu::ColorTargetState { 
                    format : surface_configuration.format,
                    blend : Some(wgpu::BlendState::REPLACE),
                    write_mask : wgpu::ColorWrites::ALL
                })]
            }),
            primitive : wgpu::PrimitiveState {
                topology : wgpu::PrimitiveTopology::TriangleList,
                strip_index_format : None,
                front_face : wgpu::FrontFace::Ccw,
                cull_mode : Some(wgpu::Face::Back),
                polygon_mode : wgpu::PolygonMode::Fill,
                unclipped_depth : false,
                conservative : false
            },
            depth_stencil : None, 
            multisample : wgpu::MultisampleState {
                count : 1,
                mask : !0,
                alpha_to_coverage_enabled : false
            },
            multiview : None
        });

        App {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_caps,
            surface_configuration,
            window_size,
            render_pipeline
        }
    }
    
    fn handle_event(&mut self, window : &winit::window::Window, event : &winit::event::Event<()>, control_flow : &mut winit::event_loop::ControlFlow) {
        match event {
            winit::event::Event::WindowEvent{ref event, window_id} => {
                if *window_id == window.id() { match event {
                    winit::event::WindowEvent::CloseRequested => *control_flow = winit::event_loop::ControlFlow::Exit,
                    winit::event::WindowEvent::Resized(physical_size) => self.update_surface_size(*physical_size),
                    _ => {} 
                }}
            },
            winit::event::Event::MainEventsCleared => window.request_redraw(),
            winit::event::Event::RedrawRequested(window_id) => self.render(),
            _ => {}
        }
    }

    fn update_surface_size(&mut self, new_window_size : winit::dpi::PhysicalSize<u32>) {
        if new_window_size.width > 0 && new_window_size.height > 0 {
            self.window_size = new_window_size;
            self.surface_configuration.width = new_window_size.width;
            self.surface_configuration.height = new_window_size.height;
            self.surface.configure(&self.device, &self.surface_configuration);
        }
    }

    fn update_surface(&mut self) {
        
    }
    

    fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label : Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label : Some("Render Pass"),
                color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                    view : &view,
                    resolve_target : None,
                    ops : wgpu::Operations {
                        load : wgpu::LoadOp::Clear(wgpu::Color{
                            r : 0.1,
                            g : 0.2,
                            b : 0.8,
                            a : 1.0
                        }),
                        store : true
                    }
                })],
                depth_stencil_attachment : None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
} 

#[tokio::main]

async fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
    let mut engine = App::new(&window).await;

    event_loop.run(move |event, _, control_flow| engine.handle_event(&window, &event, control_flow));
}