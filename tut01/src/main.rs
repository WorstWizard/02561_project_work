use winit::event::{Event, WindowEvent};
use winit::window::{Window, WindowBuilder};
use winit::event_loop::{EventLoop, ControlFlow};


struct HelloTriangleApp {}
impl HelloTriangleApp {

    fn new() -> HelloTriangleApp {
        HelloTriangleApp{}
    }

    fn run(&self) {
        let (window, event_loop) = HelloTriangleApp::init_window();
        HelloTriangleApp::init_vulkan();
        HelloTriangleApp::main_loop(window, event_loop);
    }

    fn init_window() -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size( winit::dpi::PhysicalSize::new(512, 512))
            .with_title("HelloTriangle")
            .build(&event_loop).expect("Window build failed!");
        (window, event_loop)
    }

    fn init_vulkan() {}

    ///The event loop hijacks the main thread, so once it closes the entire program exits.
    ///All cleanup operations should be handled either before the main loop, inside the mainloop,
    ///or in the drop function of any data moved into the closure
    fn main_loop(_window: Window, event_loop: EventLoop<()>) {
        event_loop.run(move |event,_,control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested, ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    println!("Test");
                },
                Event::MainEventsCleared => { //Main body
                    //If drawing continously, put rendering code here directly

                    //window.request_redraw() //Call if state changed and a redraw is necessary
                },
                Event::RedrawRequested(_) => { //Conditionally redraw (OS might request this too)
                },
                _ => ()
            }
        });
    }
}

fn main() {
    let app = HelloTriangleApp::new();

    app.run(); //Run is the last method to run, as it closes the main thread!
}