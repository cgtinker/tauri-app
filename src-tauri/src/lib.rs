// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindow, Window, WindowEvent};
use wgpu::{
    CurrentSurfaceTexture,
    rwh::{HasRawDisplayHandle, HasRawWindowHandle},
};

use std::{os::macos::raw::stat, sync::Mutex};

struct WgpuState<'a> {
    // put your wgpu stuff here
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: WebviewWindow,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Greet {
    name: String,
    x: i32,
    y: i32,
}

#[tauri::command]
fn greet(input: Greet) -> String {
    let id = std::thread::current().id();
    println!("new id {:?}", id);

    format!("Hello, ! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let id = std::thread::current().id();
    println!("main {:?}", id);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // println!("{:?}", window2);

            // Spawn async GPU init
            let instance = wgpu::Instance::default();

            // SAFETY: Tauri window provides valid raw handles
            let surface = unsafe {
                instance.create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::from_display_and_window(&window, &window).unwrap(),
                )
            }
            .unwrap();

            let adapter = pollster::block_on(
                instance.request_adapter(&wgpu::RequestAdapterOptions::default()),
            )
            .unwrap();

        println!("!!!!!!!!!! {}",adapter.get_info().name);

            let (device, queue) =
                pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                    .unwrap();

            let size = window.inner_size().unwrap();

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.get_capabilities(&adapter).formats[0],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 1,
            };

            surface.configure(&device, &config);

            let state = WgpuState {
                surface,
                device,
                queue,
                window,
            };

            app.manage(state);
            Ok(())
            // use instance + handle to create surface
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            RunEvent::MainEventsCleared => {
                let state = app_handle.state::<WgpuState>();
                let frame = state.surface.get_current_texture();
                if let CurrentSurfaceTexture::Success(texture) = frame {
                    let view = texture.texture.create_view(&Default::default());

                    let mut encoder = state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                    {
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                    }

                    state.queue.submit(Some(encoder.finish()));

                    texture.present();
                }
            }
            _ => {}
        });
}
