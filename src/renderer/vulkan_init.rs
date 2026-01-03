use std::sync::Arc;
use vulkano::{
    Version, VulkanLibrary,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    swapchain::Surface,
};
use winit::event_loop::EventLoop;

/// Initializes Vulkan instance with required extensions for window rendering
pub fn create_instance(event_loop: &EventLoop<()>) -> Arc<Instance> {
    let library = VulkanLibrary::new().unwrap();

    // The first step of any Vulkan program is to create an instance.
    //
    // When we create an instance, we have to pass a list of extensions that we want to enable.
    //
    // All the window-drawing functionalities are part of non-core extensions that we need to
    // enable manually. To do so, we ask `Surface` for the list of extensions required to draw
    // to a window.
    let required_extensions = Surface::required_extensions(event_loop).unwrap();

    // Now creating the instance.
    Instance::new(
        library,
        InstanceCreateInfo {
            // Enable enumerating devices that use non-conformant Vulkan implementations.
            // (e.g. MoltenVK)
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .unwrap()
}

/// Selects the best physical device and creates a logical device with a graphics queue
pub fn create_device(
    instance: Arc<Instance>,
    event_loop: &EventLoop<()>,
) -> (Arc<Device>, Arc<Queue>) {
    // Choose device extensions that we're going to use. In order to present images to a
    // surface, we need a `Swapchain`, which is provided by the `khr_swapchain` extension.
    let mut device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    // We then choose which physical device to use. First, we enumerate all the available
    // physical devices, then apply filters to narrow them down to those that can support our
    // needs.
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| {
            // For this example, we require at least Vulkan 1.3, or a device that has the
            // `khr_dynamic_rendering` extension available.
            p.api_version() >= Version::V1_3 || p.supported_extensions().khr_dynamic_rendering
        })
        .filter(|p| {
            // Some devices may not support the extensions or features that your application,
            // or report properties and limits that are not sufficient for your application.
            // These should be filtered out here.
            p.supported_extensions().contains(&device_extensions)
        })
        .filter_map(|p| {
            // For each physical device, we try to find a suitable queue family that will
            // execute our draw commands.
            //
            // Devices can provide multiple queues to run commands in parallel (for example a
            // draw queue and a compute queue), similar to CPU threads. This is something you
            // have to have to manage manually in Vulkan. Queues of the same type belong to the
            // same queue family.
            //
            // Here, we look for a single queue family that is suitable for our purposes. In a
            // real-world application, you may want to use a separate dedicated transfer queue
            // to handle data transfers in parallel with graphics operations. You may also need
            // a separate queue for compute operations, if your application uses those.
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    // We select a queue family that supports graphics operations. When drawing
                    // to a window surface, as we do in this example, we also need to check
                    // that queues in this queue family are capable of presenting images to the
                    // surface.
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.presentation_support(i as u32, event_loop).unwrap()
                })
                // The code here searches for the first queue family that is suitable. If none
                // is found, `None` is returned to `filter_map`, which disqualifies this
                // physical device.
                .map(|i| (p, i as u32))
        })
        // All the physical devices that pass the filters above are suitable for the
        // application. However, not every device is equal, some are preferred over others.
        // Now, we assign each physical device a score, and pick the device with the lowest
        // ("best") score.
        //
        // In this example, we simply select the best-scoring device to use in the application.
        // In a real-world setting, you may want to use the best-scoring device only as a
        // "default" or "recommended" device, and let the user choose the device themself.
        .min_by_key(|(p, _)| {
            // We assign a lower score to device types that are likely to be faster/better.
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        })
        .expect("no suitable physical device found");

    // Some little debug infos.
    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    // If the selected device doesn't have Vulkan 1.3 available, then we need to enable the
    // `khr_dynamic_rendering` extension manually. This extension became a core part of Vulkan
    // in version 1.3 and later, so it's always available then and it does not need to be
    // enabled. We can be sure that this extension will be available on the selected physical
    // device, because we filtered out unsuitable devices in the device selection code above.
    if physical_device.api_version() < Version::V1_3 {
        device_extensions.khr_dynamic_rendering = true;
    }

    // Now initializing the device. This is probably the most important object of Vulkan.
    //
    // An iterator of created queues is returned by the function alongside the device.
    let (device, mut queues) = Device::new(
        // Which physical device to connect to.
        physical_device,
        DeviceCreateInfo {
            // The list of queues that we are going to use. Here we only use one queue, from
            // the previously chosen queue family.
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],

            // A list of optional features and extensions that our program needs to work
            // correctly. Some parts of the Vulkan specs are optional and must be enabled
            // manually at device creation. In this example the only things we are going to
            // need are the `khr_swapchain` extension that allows us to draw to a window, and
            // `khr_dynamic_rendering` if we don't have Vulkan 1.3 available.
            enabled_extensions: device_extensions,

            // In order to render with Vulkan 1.3's dynamic rendering, we need to enable it
            // here. Otherwise, we are only allowed to render with a render pass object, as in
            // the standard triangle example. The feature is required to be supported by the
            // device if it supports Vulkan 1.3 and higher, or if the `khr_dynamic_rendering`
            // extension is available, so we don't need to check for support.
            enabled_features: DeviceFeatures {
                dynamic_rendering: true,
                ..DeviceFeatures::empty()
            },

            ..Default::default()
        },
    )
    .unwrap();

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. We
    // only use one queue in this example, so we just retrieve the first and only element of
    // the iterator.
    let queue = queues.next().unwrap();

    (device, queue)
}

/// Initializes Vulkan and returns instance, device, and queue
pub fn initialize_vulkan(event_loop: &EventLoop<()>) -> (Arc<Instance>, Arc<Device>, Arc<Queue>) {
    let instance = create_instance(event_loop);
    let (device, queue) = create_device(instance.clone(), event_loop);
    (instance, device, queue)
}
