use winit::event::{Event, WindowEvent};
use winit::window::{Window, WindowBuilder};
use winit::event_loop::{EventLoop, ControlFlow};

use erupt::{vk, {EntryLoader, InstanceLoader, DeviceLoader, ExtendableFrom}, utils::{surface}, cstr};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};
use std::collections::HashSet;

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

struct VulkanApp { //Members dropped in declared order. So they must be placed in opposite order of references
    device: Box<DeviceLoader>,
    surface: vk::SurfaceKHR,
    messenger: vk::DebugUtilsMessengerEXT,
    instance: Box<InstanceLoader>,
    entry: Box<EntryLoader>,
}
impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            if !self.messenger.is_null() {
                self.instance.destroy_debug_utils_messenger_ext(self.messenger, None)
            }
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
        println!("VulkanApp dropped succesfully");
    }
}
fn init_vulkan(window: &Window) -> VulkanApp {
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

    //// Application info
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

    //// Instance info & debug messenger
    let mut messenger_info = init_debug_messenger_info();
    let mut instance_info = vk::InstanceCreateInfoBuilder::new()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions);
    if VALIDATION_ENABLED {
        instance_info = instance_info
            .enabled_layer_names(&VALIDATION_LAYERS)
            .extend_from(&mut messenger_info);
    }
    
    //// Instance created
    let instance = Box::new(unsafe {InstanceLoader::new(&entry, &instance_info)}.expect("Failed to create Vulkan instance!"));
    // Messenger attached
    let messenger = if VALIDATION_ENABLED {
        unsafe {instance.create_debug_utils_messenger_ext(&messenger_info, None)}.unwrap()
    } else {
        vk::DebugUtilsMessengerEXT::default()
    };

    //// Window surface creation
    let surface = unsafe { surface::create_surface(&instance, &window, None) }.unwrap();


    //// Physical device and queues
    const GRAPHICS_Q_IDX: usize = 0;
    const PRESENT_Q_IDX: usize = 1;
    let (physical_device, queue_family_indices) = {
        fn find_queue_families(device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR, instance: &InstanceLoader) -> Option<[u32; 2]> {
            let queue_family_properties = unsafe{instance.get_physical_device_queue_family_properties(*device, None)};
            let mut indices = [0; 2];
            let mut found_queues = [false; 2];
            'outer:
            for (i, queue_family) in queue_family_properties.iter().enumerate() {
                if !found_queues[GRAPHICS_Q_IDX] && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    indices[GRAPHICS_Q_IDX] = i as u32; //Graphics queue found, look for present queue (probably the same)
                    found_queues[GRAPHICS_Q_IDX] = true;
                }
                if !found_queues[PRESENT_Q_IDX] && unsafe {instance.get_physical_device_surface_support_khr(*device, i as u32, *surface)}.unwrap() {
                    indices[PRESENT_Q_IDX] = i as u32; //Graphics queue found, look for present queue (probably the same)
                    found_queues[PRESENT_Q_IDX] = true;
                }
                for queue_found in found_queues {
                    if !queue_found {break 'outer}
                }
                return Some(indices) //Only reached if the above for loop does not break
            }
            None
        }

        let devices = unsafe {instance.enumerate_physical_devices(None)}.unwrap();
        if devices.len() == 0 {panic!("No devices with Vulkan support!")}
        fn is_device_suitable(device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR, instance: &InstanceLoader) -> bool {
            let device_properties = unsafe{instance.get_physical_device_properties(*device)};
            let device_features = unsafe{instance.get_physical_device_features(*device)};
            println!("Device name: {}", unsafe{CStr::from_ptr(&(device_properties.device_name[0]) as *const c_char)}.to_string_lossy());
            return device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && device_features.geometry_shader == vk::TRUE
                && if let Some(_) = find_queue_families(device, surface, instance) {true} else {false}
                
        }
        fn rate_device_suitability(device_properties: &vk::PhysicalDeviceProperties, device_features: &vk::PhysicalDeviceFeatures) -> u32 {
            let mut score = 0;
            if device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {score += 1000}
            score += device_properties.limits.max_image_dimension2_d;
            if device_features.geometry_shader == vk::FALSE {return 0}
            return score
        }
    
        let physical_device = devices.into_iter().max_by_key(
            |device| {
                let device_properties = unsafe{instance.get_physical_device_properties(*device)};
                let device_features = unsafe{instance.get_physical_device_features(*device)};
                rate_device_suitability(&device_properties, &device_features)
            }
        ).expect("No devices could be found!");
        if !is_device_suitable(&physical_device, &surface, &instance) {panic!("No suitable GPU found!")}
        let queue_family_indices = find_queue_families(&physical_device, &surface, &instance).unwrap();
        
        (physical_device, queue_family_indices)
    };
    
    //// Logical device
    let unique_queue_family_indices: Vec<u32> = HashSet::from(queue_family_indices.clone()).into_iter().collect();
    let device_queue_infos: &[vk::DeviceQueueCreateInfoBuilder] = &unique_queue_family_indices.into_iter().map(|index| {
        vk::DeviceQueueCreateInfoBuilder::new()
        .queue_family_index(index)
        .queue_priorities(&[1.0])
    }).collect::<Vec<vk::DeviceQueueCreateInfoBuilder>>().into_boxed_slice();
    
    let device_features = vk::PhysicalDeviceFeatures::default();
    let mut device_create_info = vk::DeviceCreateInfoBuilder::new()
        .queue_create_infos(device_queue_infos)
        .enabled_features(&device_features);
    if VALIDATION_ENABLED {
        device_create_info = device_create_info.enabled_layer_names(&VALIDATION_LAYERS);
    }
    let device = Box::new(unsafe{DeviceLoader::new(&instance, physical_device, &device_create_info)}.expect("Failed to create logical device!"));

    //// Queue handles
    let graphics_queue = unsafe{device.get_device_queue(queue_family_indices[GRAPHICS_Q_IDX], 0)};
    let present_queue = unsafe{device.get_device_queue(queue_family_indices[PRESENT_Q_IDX], 0)};

    VulkanApp {
        entry,
        instance,
        device,
        messenger,
        surface,
    }
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
    let vulkan_app = init_vulkan(&window);

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

                println!("Exiting event loop, should drop application");
                &vulkan_app; //App referred to in closure, it is dropped once the scope closes
            }
            _ => ()
        }
    });
}