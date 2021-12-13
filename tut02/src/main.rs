use winit::event::{Event, WindowEvent};
use winit::window::{Window, WindowBuilder};
use winit::event_loop::{EventLoop, ControlFlow};

use erupt::{vk, {EntryLoader, InstanceLoader}, utils::{surface}, vk1_0};
use std::ffi::CString;
use std::ptr;

const HEIGHT: u32 = 512;
const WIDTH: u32 = 512;
const APP_TITLE: &str = "HelloTriangle";

struct HelloTriangleApp {}
impl HelloTriangleApp {

    fn new() -> HelloTriangleApp {
        HelloTriangleApp {}
    }

    fn init_vulkan(window: &Window) -> (Box<EntryLoader>, Box<InstanceLoader>) {
        let entry = Box::new(EntryLoader::new().unwrap());

        let app_name = CString::new("Hello Triangle").unwrap();
        let engine_name = CString::new("No Engine").unwrap();

        let app_info = vk::ApplicationInfoBuilder::new()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0,1,0,0))
            .engine_name(&engine_name)
            .engine_version(vk::API_VERSION_1_0)
            .api_version(vk::API_VERSION_1_0);

        let instance_extensions = surface::enumerate_required_extensions(window).unwrap();
        //instance_extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
            
        let instance_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions);
        
        let instance = Box::new(unsafe {InstanceLoader::new(&entry, &instance_info)}.expect("Failed to create Vulkan instance!"));

        (entry, instance)
    }

    fn init_window() -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size( winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
            .with_title(APP_TITLE)
            .build(&event_loop).expect("Window build failed!");
        (window, event_loop)
    }

    fn run(&self) {
        let (window, event_loop) = HelloTriangleApp::init_window();
        let (entry, instance) = HelloTriangleApp::init_vulkan(&window);
        HelloTriangleApp::main_loop(instance, window, event_loop);
    }

    ///The event loop hijacks the main thread, so once it closes the entire program exits.
    ///All cleanup operations should be handled either before the main loop, inside the mainloop,
    ///or in the drop function of any data moved into the closure
    fn main_loop(instance: Box<InstanceLoader>, _window: Window, event_loop: EventLoop<()>) {
        event_loop.run(move |event,_,control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested, ..
                } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::MainEventsCleared => { //Main body
                    //If drawing continously, put rendering code here directly

                    //window.request_redraw() //Call if state changed and a redraw is necessary
                },
                Event::RedrawRequested(_) => { //Conditionally redraw (OS might request this too)
                },
                Event::LoopDestroyed => unsafe {
                    instance.destroy_instance(None);
                    println!("Clean exit");
                }
                _ => ()
            }
        });
    }
}

fn main() {
    let app = HelloTriangleApp::new();

    app.run(); //Run is the last method to run, as it closes the main thread!
}