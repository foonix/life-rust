use std::fmt::Display;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::device::{Queue, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    StandardMemoryAllocator,
};
use vulkano::{DeviceSize, Validated, VulkanError, VulkanLibrary};

pub mod game_impls;

pub trait Gol {
    fn from_slice(ize: usize, vec: &[bool]) -> Self
    where
        Self: Sized;
    fn to_vec(&self) -> Vec<bool>;
    fn to_next(&self) -> Box<dyn Gol>;
    fn print(&self);
}

// Common Vulkan ojects for allocating compute or graphics resources.
pub struct VulkanContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
}

impl VulkanContext {
    // create a vulkan context using the first available GPU
    pub fn try_create() -> Result<VulkanContext, Validated<VulkanError>> {
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(library, InstanceCreateInfo::default())?;

        for (i, g) in instance.enumerate_physical_device_groups()?.enumerate() {
            println!("Group {} {}", i, g.subset_allocation);

            for d in g.physical_devices.iter() {
                println!("  {}", d.properties().device_name);
                for family in d.queue_family_properties() {
                    println!("    {:?}", family);
                }
            }
        }
        let physical_device = instance
            .enumerate_physical_devices()?
            .next()
            .expect("no devices available");

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .expect("couldn't find a graphical queue family")
            as u32;

        println!(
            "Using device: {:?} queue family: {:?}",
            physical_device.properties().device_name,
            queue_family_index
        );

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                // here we pass the desired queue family to use by index
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );

        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(device.clone(), Default::default());

        let context = VulkanContext {
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        };

        Ok(context)
    }

    fn compute_buffer_uninit(&self, size: usize) -> Subbuffer<[u32]> {
        Buffer::new_slice::<u32>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            size as DeviceSize,
        )
        .expect("failed to create buffer")
    }

    fn compute_buffer_from_iter(
        &self,
        content: impl ExactSizeIterator<Item = u32>,
    ) -> Subbuffer<[u32]> {
        Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            content,
        )
        .expect("failed to create buffer")
    }

    fn uniform_buffer_from_iter(
        &self,
        content: impl ExactSizeIterator<Item = u32>,
    ) -> Subbuffer<[u32]> {
        Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            content,
        )
        .expect("failed to create buffer")
    }
}

impl Display for VulkanContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Using device: {:?} queue family: {:?}",
            self.device.physical_device().properties().device_name,
            self.queue.queue_family_index()
        )
    }
}
