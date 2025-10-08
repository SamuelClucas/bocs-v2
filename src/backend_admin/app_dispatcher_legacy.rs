use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;
use std::sync::Arc;

use crate::backend_admin::state_legacy::State;

/// Setup for logical App struct \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n
#[derive(Default)]
pub struct App {
    state: Option<State>,
    proxy: Option<EventLoopProxy<State>>,
    aspect_ratio: f32,
    size: Option<PhysicalSize<u32>>,
    minimum_size: PhysicalSize<u32>,
    maximum_size: Option<PhysicalSize<u32>>
}

impl App  {
    pub fn new (fun: impl FnOnce()-> EventLoopProxy<State>) -> App {
        App {
            state: None,
            proxy:  Some(fun()), // smuggle proxy into app using move closure for downstream requests
            aspect_ratio: 16.0 / 9.0, // width/height,
            size: None,
            minimum_size: PhysicalSize::new(740, 360), // 40x a_r
            maximum_size: None
        }
    }}


/// implements ApplicationHandler for logical App
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { // ran once on init
        let primary_monitor = event_loop.primary_monitor().expect("No primary monitor\n");

        let physical_dims = primary_monitor.size();
        let max_physical_width = physical_dims.width as f32;
        let max_physical_height = max_physical_width as f32 / self.aspect_ratio;

        self.maximum_size = Some(winit::dpi::PhysicalSize::new(
            max_physical_width as u32, 
            max_physical_height as u32
        ));

        self.size = self.maximum_size;
        
        let time_step_width = primary_monitor.refresh_rate_millihertz() // use later for vsync
        .map(|r| 1.0 / (r as f32 / 1000.0))
        .unwrap_or_else(|| {
            println!("Could not determine refresh rate, defaulting to 16.67ms timestep\n");
            1.0 / 60.0
        });

        if let Some(size) = self.size {
        // window: logical representation of gpu output, managed by system os
            let window_attributes = Window::default_attributes()
                    .with_title("📦")
                    .with_blur(true)
                    .with_inner_size(size);
            let window = Arc::new(event_loop
                .create_window(window_attributes)
                .expect("\x1b[1;31mError creating window\x1b[0m\n"));
            window.set_min_inner_size(Some(self.minimum_size));
            window.set_max_inner_size(self.maximum_size);
            // Need async context for requests using pollset
            // injected back into app through user event
            if let Some(prx) = self.proxy.take() {
                let state = pollster::block_on(State::new(window.clone())).expect("Couldn't get state");
                self.user_event(&event_loop,state);
            } // end of setup: go to App::user_event() :)
        }
        else { println!("Fail on resumed()\n"); }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: winit::window::WindowId, event: WindowEvent) {
        let state_ = match &mut self.state {
            Some(s) => s,
            None => return
        };

        match event {
            WindowEvent::CursorMoved { device_id: _device_id, position } => {
                if let Some(old_pos) = state_.mouse_pos {
                    let delta = PhysicalPosition {
                        x: (position.x - old_pos.x) as f32,
                        y: (position.y - old_pos.y) as f32
                    };
                    // handle cursor move here
                    state_.world.camera.handle_rotate(delta.x, delta.y);
                    state_.mouse_pos = Some(position);
                }
                else { state_.mouse_pos = Some(position); }
            },
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(code),
                        state: key_state, 
                        ..},
                ..} => {
                    state_.handle_key(&event_loop, code, key_state.is_pressed()) // self.state, not KeyEvent::state
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(size) => {
                self.size = Some(size);
                state_.gfx_ctx.surface_configured = false;
                match size {
                    PhysicalSize{width, height: _} => {
                        let texture_height = (width as f32 / self.aspect_ratio) as u32;
                        state_.resize(width, texture_height);
                    }}
            },       
            WindowEvent::RedrawRequested => {
                match state_.render() {
                    Ok(_) => {state_.gfx_ctx.window.request_redraw();},
                    Err(e) => {
                        println!("Unable to render {}", e);
                    }
                }
            },
            _ => {()},
        }
        }
    }
