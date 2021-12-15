use winit::event::{Event, WindowEvent};
use winit::window::{Window, WindowBuilder};
use winit::event_loop::{EventLoop, ControlFlow};

use erupt::{vk, {EntryLoader, InstanceLoader, ExtendableFrom}, utils::{surface}, cstr};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};

const HEIGHT: u32 = 512;
const WIDTH: u32 = 512;
const APP_TITLE: &str = "HelloTriangle";

const VALIDATION_LAYERS: [*const c_char; 1] = [cstr!("VK_LAYER_KHRONOS_validation")];

#[cfg(debug_assertions)]
const VALIDATION_ENABLED: bool = true;
#[cfg(not(debug_assertions))]
const VALIDATION_ENABLED: bool = false;



unsafe extern "system" fn debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void
) -> vk::Bool32 {
    eprintln!("{}", CStr::from_ptr((*p_callback_data).p_message).to_string_lossy());
    vk::FALSE
}



fn init_window() -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size( winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
        .with_title(APP_TITLE)
        .build(&event_loop).expect("Window build failed!");
    (window, event_loop)
}


fn init_vulkan(window: &Window) -> (Box<EntryLoader>, Box<InstanceLoader>, vk::DebugUtilsMessengerEXT) {
    let entry = Box::new(EntryLoader::new().unwrap());

    fn check_validation_layer_support(entry: &EntryLoader) -> bool{
        let available_layers = unsafe {entry.enumerate_instance_layer_properties(None).unwrap()};
        for layer in &VALIDATION_LAYERS {
            let mut found = false;
            for layer_properties in &available_layers {
                let layer_name_ptr = &layer_properties.layer_name[0] as *const i8;
                unsafe {
                    //println!("{:?}", CStr::from_ptr(layer_name_ptr));
                    if CStr::from_ptr(layer_name_ptr) == CStr::from_ptr(*layer) {
                        found = true; break
                    }
                }
            }
            if !found {return false}
        }
        return true
    }

    if !check_validation_layer_support(&entry) {
        panic!("Validation layer requested but not available!");
    }

    let app_name = CString::new("Hello Triangle").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let app_info = vk::ApplicationInfoBuilder::new()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0,1,0,0))
        .engine_name(&engine_name)
        .engine_version(vk::API_VERSION_1_0)
        .api_version(vk::API_VERSION_1_0);

    let mut instance_extensions = surface::enumerate_required_extensions(window).unwrap();
    if VALIDATION_ENABLED {
        instance_extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
    }

    let mut messenger_info = init_debug_messenger_info();

    let mut instance_info = vk::InstanceCreateInfoBuilder::new()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions);
    if VALIDATION_ENABLED {
        instance_info = instance_info
            .enabled_layer_names(&VALIDATION_LAYERS)
            .extend_from(&mut messenger_info);
    }
    
    let instance = Box::new(unsafe {InstanceLoader::new(&entry, &instance_info)}.expect("Failed to create Vulkan instance!"));

    let messenger = if VALIDATION_ENABLED {
        unsafe {instance.create_debug_utils_messenger_ext(&messenger_info, None)}.unwrap()
    } else {
        vk::DebugUtilsMessengerEXT::default()
    };

    (entry, instance, messenger)
}


fn init_debug_messenger_info() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
    let messenger_info = vk::DebugUtilsMessengerCreateInfoEXTBuilder::new()
    .message_severity(
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE_EXT |
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT |
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT
    )
    .message_type(
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT |
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT |
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT
    )
    .pfn_user_callback(Some(debug_callback));
    messenger_info
}

fn main() {
    let (window, event_loop) = init_window();
<<<<<<< HEAD
    let (_entry, instance, messenger) = init_vulkan(&window);
=======
    let (entry, instance, messenger) = init_vulkan(&window);
>>>>>>> 38c298b4ed0df823e01e81fe0a0eede672e6afd1

    //The event loop hijacks the main thread, so once it closes the entire program exits.
    //All cleanup operations should be handled either before the main loop, inside the mainloop,
    //or in the drop function of any data moved into the closure
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

                if !messenger.is_null() {
                    instance.destroy_debug_utils_messenger_ext(messenger, None)
                }
                instance.destroy_instance(None);
                
                println!("Clean exit");
            }
            _ => ()
        }
    });
}