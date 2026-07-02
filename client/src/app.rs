use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::network::ServerMessage;
use crate::ui::{EditorState, LogLevel};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Muse — AI Character & Animation Studio")
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let surface = instance.create_surface(window.clone()).expect("Failed to create surface");
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to request adapter");
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
        .expect("Failed to request device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);
    let size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
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

    let mut egui_winit = egui_winit::State::new(
        egui::Context::default(),
        egui::viewport::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut egui_renderer =
        egui_wgpu::Renderer::new(&device, config.format, None, 1);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let ws_tx = Arc::new(Mutex::new(Some(tx)));
    let mut editor = EditorState::new(ws_tx.clone());

    let log_queue = Arc::new(Mutex::new(Vec::<(LogLevel, String)>::new()));
    let log_queue_clone = log_queue.clone();

    let state_queue = Arc::new(Mutex::new(Vec::<ServerMessage>::new()));
    let state_queue_clone = state_queue.clone();

    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            loop {
                match connect_async("ws://127.0.0.1:8081/ws").await {
                    Ok((ws_stream, _)) => {
                        {
                            let mut guard = log_queue_clone.lock().expect("log queue lock poisoned");
                            guard.push((LogLevel::Info, "[WS] connected".to_string()));
                        }
                        let (mut write, mut read) = ws_stream.split();

                        let log_queue_clone2 = log_queue_clone.clone();
                        let state_queue_clone2 = state_queue_clone.clone();
                        let read_task = async {
                            while let Some(msg) = read.next().await {
                                if let Ok(Message::Text(text)) = msg {
                                    match serde_json::from_str::<ServerMessage>(&text) {
                                        Ok(msg_enum) => {
                                            let mut q = state_queue_clone2.lock().expect("state queue lock poisoned");
                                            match &msg_enum {
                                                ServerMessage::Error { detail } => {
                                                    let mut guard = log_queue_clone2.lock().expect("log queue lock poisoned");
                                                    guard.push((
                                                        LogLevel::Err,
                                                        format!("Server: {}", detail),
                                                    ));
                                                }
                                                _ => {}
                                            }
                                            q.push(msg_enum);
                                        }
                                        Err(_) => {}
                                    }
                                }
                            }
                        };

                        let log_queue_clone3 = log_queue_clone.clone();
                        let write_task = async {
                            while let Some(to_send) = rx.recv().await {
                                if let Err(e) = write.send(Message::Text(to_send)).await {
                                    let mut guard = log_queue_clone3.lock().expect("log queue lock poisoned");
                                    guard.push((
                                        LogLevel::Err,
                                        format!("[WS] send error: {}", e),
                                    ));
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
                        {
                            let mut guard = log_queue_clone.lock().expect("log queue lock poisoned");
                            guard.push((LogLevel::Warn, format!("[WS] connection failed: {}", e)));
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
    });

    let mut frame_count = 0;
    let mut fps = 0.0;
    let mut fps_timer = std::time::Instant::now();

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                let response = egui_winit.on_window_event(&window, event);
                if response.consumed {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            config.width = physical_size.width;
                            config.height = physical_size.height;
                            surface.configure(&device, &config);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        frame_count += 1;
                        let now = std::time::Instant::now();
                        if now.duration_since(fps_timer).as_secs_f32() >= 1.0 {
                            fps = frame_count as f32 / now.duration_since(fps_timer).as_secs_f32();
                            frame_count = 0;
                            fps_timer = now;
                        }
                        editor.metrics.fps = fps;

                        {
                            let mut logs = log_queue.lock().expect("log queue lock poisoned");
                            for (lvl, l) in logs.drain(..) {
                                editor.push_log(lvl, &l);
                            }
                        }

                        {
                            let mut updates = state_queue.lock().expect("state queue lock poisoned");
                            for msg in updates.drain(..) {
                                match msg {
                                    ServerMessage::JobUpdate { job } => {
                                        match job.status.as_str() {
                                            "queued" => editor.gen_status.add_job(job.id.clone(), job.job_type.clone()),
                                            "running" => editor.gen_status.update_progress(&job.id, job.progress as f32),
                                            "completed" => editor.gen_status.complete(&job.id),
                                            "failed" => editor.gen_status.fail(&job.id, job.error.unwrap_or_default()),
                                            _ => {}
                                        }
                                    }
                                    ServerMessage::MeshGenerated { mesh, skeleton, clip: _ } => {
                                        editor.loaded_character = true;
                                        editor.character_mesh = Some(mesh);
                                        editor.character_skeleton = Some(skeleton);
                                        editor.push_log(LogLevel::Ok, "Character loaded");
                                    }
                                    ServerMessage::MotionGenerated { clip } => {
                                        editor.loaded_motion = true;
                                        editor.motion_clip = Some(clip);
                                        editor.push_log(LogLevel::Ok, "Motion loaded");
                                    }
                                    ServerMessage::Error { .. } => {}
                                }
                            }
                        }

                        let output = match surface.get_current_texture() {
                            Ok(output) => output,
                            Err(wgpu::SurfaceError::Outdated) => return,
                            Err(e) => {
                                eprintln!("Dropped frame: {:?}", e);
                                return;
                            }
                        };
                        let view = output
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let input = egui_winit.take_egui_input(&window);
                        egui_winit.egui_ctx().begin_frame(input);
                        editor.draw(egui_winit.egui_ctx());
                        let full_output = egui_winit.egui_ctx().end_frame();
                        let paint_jobs = egui_winit
                            .egui_ctx()
                            .tessellate(full_output.shapes, full_output.pixels_per_point);

                        let mut encoder = device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            },
                        );

                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [config.width, config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

                        for (id, delta) in &full_output.textures_delta.set {
                            egui_renderer
                                .update_texture(&device, &queue, *id, delta);
                        }
                        egui_renderer.update_buffers(
                            &device,
                            &queue,
                            &mut encoder,
                            &paint_jobs,
                            &screen_descriptor,
                        );

                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Render Pass"),
                                    color_attachments: &[Some(
                                        wgpu::RenderPassColorAttachment {
                                            view: &view,
                                            resolve_target: None,
                                            ops: wgpu::Operations {
                                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                                store: wgpu::StoreOp::Store,
                                            },
                                        },
                                    )],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });

                            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        }

                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        queue.submit(std::iter::once(encoder.finish()));
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
