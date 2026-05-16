use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wgpu::util::DeviceExt;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use serde::Deserialize;

mod live_editor_ui;
use live_editor_ui::{EditorState, LogLevel};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct StateUniform {
    entities: [[f32; 4]; 64],
    count: u32,
    padding: [u32; 3],
}

impl Default for StateUniform {
    fn default() -> Self {
        Self {
            entities: [[0.0; 4]; 64],
            count: 0,
            padding: [0; 3],
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ServerMessage {
    PhysicsState { x: Vec<f32>, y: Vec<f32>, z: Vec<f32> },
    ShaderUpdate { wgsl: String },
    Error { detail: String },
}

struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    shader_module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
}

impl Renderer {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
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
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().filter(|f| f.is_srgb()).next().unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[StateUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = Self::create_pipeline(&device, &pipeline_layout, &shader_module, config.format);

        Self {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            bind_group,
            uniform_buffer,
            shader_module,
            bind_group_layout,
            pipeline_layout,
        }
    }

    fn create_pipeline(device: &wgpu::Device, layout: &wgpu::PipelineLayout, module: &wgpu::ShaderModule, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn update_shader(&mut self, wgsl: &str) {
        let new_shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dynamic Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });
        
        self.render_pipeline = Self::create_pipeline(&self.device, &self.pipeline_layout, &new_shader, self.config.format);
        self.shader_module = new_shader;
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let window = Arc::new(WindowBuilder::new().with_title("Rationalist").build(&event_loop).unwrap());
    let mut renderer = Renderer::new(window.clone()).await;

    // Egui setup
    let mut egui_winit = egui_winit::State::new(
        egui::Context::default(),
        egui::viewport::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut egui_renderer = egui_wgpu::Renderer::new(&renderer.device, renderer.config.format, None, 1);

    let state = Arc::new(Mutex::new(StateUniform::default()));
    let state_clone = state.clone();
    
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let ws_tx = Arc::new(Mutex::new(Some(tx)));
    let mut editor = EditorState::new(ws_tx.clone());

    let new_wgsl = Arc::new(Mutex::new(None::<String>));
    let new_wgsl_clone = new_wgsl.clone();
    let log_queue = Arc::new(Mutex::new(Vec::<(LogLevel, String)>::new()));
    let log_queue_clone = log_queue.clone();

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            loop {
                match connect_async("ws://127.0.0.1:8080/ws").await {
                    Ok((ws_stream, _)) => {
                        log_queue_clone.lock().unwrap().push((LogLevel::Info, "[WS] connected".to_string()));
                        let (mut write, mut read) = ws_stream.split();
                        
                        let read_task = async {
                            while let Some(msg) = read.next().await {
                                if let Ok(Message::Text(text)) = msg {
                                    match serde_json::from_str::<ServerMessage>(&text) {
                                        Ok(msg_enum) => {
                                            match msg_enum {
                                                ServerMessage::PhysicsState { x, y, z } => {
                                                    let mut lock = state_clone.lock().unwrap();
                                                    let count = x.len().min(64);
                                                    lock.count = count as u32;
                                                    for i in 0..count {
                                                        lock.entities[i][0] = x[i];
                                                        lock.entities[i][1] = y[i];
                                                        lock.entities[i][2] = z[i];
                                                        lock.entities[i][3] = 0.0;
                                                    }
                                                }
                                                ServerMessage::ShaderUpdate { wgsl } => {
                                                    log_queue_clone.lock().unwrap().push((LogLevel::Ok, "shader update received".to_string()));
                                                    *new_wgsl_clone.lock().unwrap() = Some(wgsl);
                                                }
                                                ServerMessage::Error { detail } => {
                                                    log_queue_clone.lock().unwrap().push((LogLevel::Err, format!("Compiler: {}", detail)));
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            // Handle raw JSON errors or other message types if needed
                                        }
                                    }
                                }
                            }
                        };

                        let write_task = async {
                            while let Some(to_send) = rx.recv().await {
                                if let Err(e) = write.send(Message::Text(to_send)).await {
                                    log_queue_clone.lock().unwrap().push((LogLevel::Err, format!("[WS] send error: {}", e)));
                                    break;
                                }
                            }
                        };

                        tokio::select! {
                            _ = read_task => {},
                            _ = write_task => {},
                        }
                    }
                    Err(e) => {
                        log_queue_clone.lock().unwrap().push((LogLevel::Warn, format!("[WS] connection failed: {}", e)));
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
    });

    let mut _last_frame_time = std::time::Instant::now();
    let mut frame_count = 0;
    let mut fps = 0.0;
    let mut fps_timer = std::time::Instant::now();

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                let response = egui_winit.on_window_event(&window, event);
                if response.consumed {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            renderer.config.width = physical_size.width;
                            renderer.config.height = physical_size.height;
                            renderer.surface.configure(&renderer.device, &renderer.config);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // FPS calculation
                        frame_count += 1;
                        let now = std::time::Instant::now();
                        if now.duration_since(fps_timer).as_secs_f32() >= 1.0 {
                            fps = frame_count as f32 / now.duration_since(fps_timer).as_secs_f32();
                            frame_count = 0;
                            fps_timer = now;
                        }
                        editor.metrics.fps = fps;

                        // Check for new shaders
                        if let Some(wgsl) = new_wgsl.lock().unwrap().take() {
                            renderer.update_shader(&wgsl);
                            editor.push_log(LogLevel::Ok, "pipeline updated");
                        }

                        // Flush logs
                        {
                            let mut logs = log_queue.lock().unwrap();
                            for (lvl, l) in logs.drain(..) {
                                if let LogLevel::Err = lvl {
                                    editor.error_msg = Some(l.clone());
                                } else if let LogLevel::Ok = lvl {
                                    editor.error_msg = None;
                                }
                                editor.push_log(lvl, &l);
                            }
                        }

                        let uniform = *state.lock().unwrap();
                        renderer.queue.write_buffer(&renderer.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));

                        // Update uniforms tab data
                        editor.uniforms = vec![
                            ("state.count".to_string(), format!("{}", uniform.count)),
                            ("resolution".to_string(), format!("{}x{}", renderer.config.width, renderer.config.height)),
                            ("entities".to_string(), format!("{}", uniform.count)),
                        ];
                        if uniform.count > 0 {
                            editor.uniforms.push(("state.entities[0]".to_string(), format!("({:.2}, {:.2}, {:.2})", uniform.entities[0][0], uniform.entities[0][1], uniform.entities[0][2])));
                        }

                        let output = match renderer.surface.get_current_texture() {
                            Ok(output) => output,
                            Err(wgpu::SurfaceError::Outdated) => return,
                            Err(e) => {
                                eprintln!("Dropped frame: {:?}", e);
                                return;
                            }
                        };
                        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                        // Egui rendering
                        let input = egui_winit.take_egui_input(&window);
                        egui_winit.egui_ctx().begin_frame(input);
                        editor.draw(egui_winit.egui_ctx());
                        let full_output = egui_winit.egui_ctx().end_frame();
                        let paint_jobs = egui_winit.egui_ctx().tessellate(full_output.shapes, full_output.pixels_per_point);

                        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
                        
                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [renderer.config.width, renderer.config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

                        for (id, delta) in &full_output.textures_delta.set {
                            egui_renderer.update_texture(&renderer.device, &renderer.queue, *id, delta);
                        }
                        egui_renderer.update_buffers(&renderer.device, &renderer.queue, &mut encoder, &paint_jobs, &screen_descriptor);

                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            render_pass.set_pipeline(&renderer.render_pipeline);
                            render_pass.set_bind_group(0, &renderer.bind_group, &[]);
                            render_pass.draw(0..3, 0..1);

                            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        }

                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        renderer.queue.submit(std::iter::once(encoder.finish()));
                        output.present();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    pollster::block_on(run());
}
