use std::fmt::Display;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, CopyImageToBufferInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::device::{Queue, QueueFlags};
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    StandardMemoryAllocator,
};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError, VulkanLibrary};

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

    fn uninitialized_image(&self, extent: [u32; 3]) -> Arc<Image> {
        Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8_UINT,
                extent,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::STORAGE,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap()
    }

    fn image_from_iter(
        &self,
        extents: [u32; 3],
        content: impl ExactSizeIterator<Item = u8>,
    ) -> Arc<Image> {
        // allocate and populate staging buffer
        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            content,
        )
        .expect("failed to create transfer buffer");

        assert!(buffer.size() == (extents[0] * extents[1] * extents[2]) as u64);

        // create image
        let image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8_UINT,
                extent: extents,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::STORAGE,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        // command buffer to copy staging to image
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                buffer.clone(),
                image.clone(),
            ))
            .unwrap();

        // submit buffer and join
        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        image
    }

    fn buffer_from_image(&self, image: &Arc<Image>) -> Subbuffer<[u8]> {
        let src_extent = image.extent();
        // assuming R8
        let image_size = src_extent[0] * src_extent[1] * src_extent[2];

        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            (0..image_size).map(|_| 0u8),
        )
        .expect("failed to create transfer buffer");

        // command buffer to copy to image to buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                image.clone(),
                buffer.clone(),
            ))
            .unwrap();

        // submit buffer and join
        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        buffer
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
