use ash::vk;
use super::*;

pub(super) fn physical_device_score(properties: &vk::PhysicalDeviceProperties) -> usize
{
    let mut score = 0;
    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    {
        score += 1000;
    }

    score
}

pub(super) fn create(instance: &vulkan::Instance) -> vk::PhysicalDevice 
{

    use std::collections::BTreeMap;

    let mut device_scores: BTreeMap<usize, usize> = BTreeMap::new();

    let physical_devices = unsafe {
        instance.instance.enumerate_physical_devices()
            .expect("failed to query physical devices")
    };

    unsafe 
    {
        for (n, &device) in physical_devices.iter().enumerate()
        {
            let score = physical_device_score(
                &instance.instance.get_physical_device_properties(device)
            );
            device_scores.insert(score, n);
        }
    }

    physical_devices[device_scores.last_entry().unwrap().remove()]
}

pub(super) fn get_queue_families(
    instance: &vulkan::Instance, 
    device: vk::PhysicalDevice, 
    surface: vk::SurfaceKHR
) -> Result<QueueFamilyIndices, ()>
{
    let mut present_queue: Option<u32> = None;
    let mut graphics_queue: Option<u32> = None;
    let mut transfer_queue: Option<u32> = None;

    let properties = unsafe{instance.instance.get_physical_device_queue_family_properties(device)};

    for (i, family) in properties.iter().enumerate() 
    {
        let graphics_support = family.queue_flags & vk::QueueFlags::GRAPHICS != vk::QueueFlags::empty();
        let transfer_support = family.queue_flags & vk::QueueFlags::TRANSFER != vk::QueueFlags::empty();
        let present_support = unsafe{
            instance.surface.get_physical_device_surface_support(device, i as _, surface)
        }.unwrap();

        if transfer_support {
            transfer_queue = Some(i as u32);
        }

        if graphics_support && Some(i as u32) != transfer_queue {
            graphics_queue = Some(i as u32);
        }

        if present_support && Some(i as u32) != transfer_queue {
            transfer_queue = Some(i as u32);
        }
    }

    if graphics_queue.is_none() && transfer_queue.is_some() {
        graphics_queue = transfer_queue;
    }

    if present_queue.is_none() && transfer_queue.is_some() {
        present_queue = transfer_queue;
    }

    if let Some(gq) = graphics_queue && let Some(pq) = present_queue && let Some(tq) = transfer_queue {
        Ok(QueueFamilyIndices {graphics: gq, present: pq, transfer: tq})
    } else {
        Err(())
    }
}
pub(super) struct QueueFamilyIndices {
    pub(super) graphics: u32,
    pub(super) present: u32,
    pub(super) transfer: u32
}

impl QueueFamilyIndices 
{
    pub(super) fn iter(&self) -> [u32; 3]
    {
        [self.graphics, self.present, self.transfer]
    }
}