use std::sync::Arc;

use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::{layout::DescriptorSetLayout, PersistentDescriptorSet, WriteDescriptorSet},
    image::{view::ImageView, Image},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    sync::{self, GpuFuture},
};

use crate::{Gol, VulkanContext};

pub struct GameState {
    size: (usize, usize),
    context: Arc<VulkanContext>,
    game_state: Arc<Image>,
    compute_pipeline: Arc<ComputePipeline>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
    bounds_buffer: Subbuffer<[u32]>,
}

impl GameState {
    pub fn from_random(context: Arc<VulkanContext>, size: usize) -> GameState {
        let game_state = context.image_from_iter(
            [size as u32, size as u32, 1],
            (0..size * size).map(|_| rand::random::<u8>() & 1),
        );

        let bounds_buffer =
            context.uniform_buffer_from_iter([size as u32, size as u32].into_iter());

        let (compute_pipeline, descriptor_set_layout) = Self::create_pipeline(context.clone());

        GameState {
            size: (size, size),
            context: context.clone(),
            game_state,
            compute_pipeline,
            descriptor_set_layout,
            bounds_buffer,
        }
    }

    fn create_pipeline(
        context: Arc<VulkanContext>,
    ) -> (Arc<ComputePipeline>, Arc<DescriptorSetLayout>) {
        let shader = cs::load(context.device.clone()).expect("failed to create shader module");
        let cs = shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(cs);
        let layout = PipelineLayout::new(
            context.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(context.device.clone())
                .unwrap(),
        )
        .unwrap();

        let descriptor_set_layouts = layout.set_layouts();
        let descriptor_set_layout_index = 0;
        let descriptor_set_layout = descriptor_set_layouts
            .get(descriptor_set_layout_index)
            .unwrap();

        (
            ComputePipeline::new(
                context.device.clone(),
                None,
                ComputePipelineCreateInfo::stage_layout(stage, layout.clone()),
            )
            .expect("failed to create compute pipeline"),
            descriptor_set_layout.clone(),
        )
    }
}

impl Gol for GameState {
    fn from_slice(size: usize, slice: &[bool]) -> Self
    where
        Self: Sized,
    {
        let context = Arc::new(VulkanContext::try_create().unwrap());

        let game_state = context.image_from_iter(
            [size as u32, size as u32, 1],
            slice.iter().map(|b| if *b { 1u8 } else { 0u8 }),
        );

        let bounds_buffer =
            context.uniform_buffer_from_iter([size as u32, size as u32].into_iter());

        let (compute_pipeline, descriptor_set_layout) = Self::create_pipeline(context.clone());

        GameState {
            size: (size, size),
            context: context.clone(),
            game_state,
            compute_pipeline,
            descriptor_set_layout,
            bounds_buffer,
        }
    }

    fn to_vec(&self) -> Vec<bool> {
        let buffer_content = self.context.buffer_from_image(&self.game_state);
        let binding = buffer_content.read().unwrap();
        Vec::from_iter(binding.iter().map(|x| *x > 0u8))
    }

    fn to_next(&self) -> Box<dyn Gol> {
        let next_state =
            self.context
                .uninitialized_image([self.size.0 as u32, self.size.1 as u32, 1]);

        let view_previous = ImageView::new_default(self.game_state.clone()).unwrap();
        let view_next = ImageView::new_default(next_state.clone()).unwrap();

        let descriptor_set = PersistentDescriptorSet::new(
            &self.context.descriptor_set_allocator,
            self.descriptor_set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, self.bounds_buffer.clone()),
                WriteDescriptorSet::image_view(1, view_previous.clone()),
                WriteDescriptorSet::image_view(2, view_next.clone()),
            ], // 0 is the binding
            [],
        )
        .unwrap();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.context.command_buffer_allocator,
            self.context.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let work_group_counts = [
            (self.size.0 as u32 - 1) / 8 + 1,
            (self.size.1 as u32 - 1) / 8 + 1,
            1,
        ];

        command_buffer_builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .dispatch(work_group_counts)
            .unwrap();

        let command_buffer = command_buffer_builder.build().unwrap();

        let future = sync::now(self.context.device.clone())
            .then_execute(self.context.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        Box::new(GameState {
            size: self.size,
            context: self.context.clone(),
            game_state: next_state,
            compute_pipeline: self.compute_pipeline.clone(),
            descriptor_set_layout: self.descriptor_set_layout.clone(),
            bounds_buffer: self.bounds_buffer.clone(),
        })
    }

    fn print(&self) {
        for (i, is_alive) in self.to_vec().iter().enumerate() {
            print!("{}", if *is_alive { "1" } else { "0" });
            if (i + 1) % (self.size.0) == 0 {
                println!();
            }
        }
    }
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(set = 0, binding = 0) uniform game_params {
                uvec2 game_size;
            } params;

            layout(set = 0, binding = 1, r8ui) uniform readonly uimage2D src;

            layout(set = 0, binding = 2, r8ui) uniform writeonly uimage2D dest;

            uint coords_to_idx(uvec2 coords)
            {
                uvec2 wrapped_coords = coords % params.game_size;
                uint ret = wrapped_coords.y * params.game_size.y + wrapped_coords.x;
                return ret;
            }

            uint is_alive(ivec2 coords)
            {
                return imageLoad(src, ivec2(coords % params.game_size)).x;
            }

            void main() {
                if(gl_GlobalInvocationID.x >= params.game_size.x || gl_GlobalInvocationID.y >= params.game_size.y)
                {
                    return;
                }

                ivec2 id = ivec2(params.game_size + gl_GlobalInvocationID.xy);
                ivec2 id_abs = ivec2(gl_GlobalInvocationID.xy);

                uint total = 0;
                total += is_alive(id + ivec2(-1, -1));
                total += is_alive(id + ivec2(0, -1));
                total += is_alive(id + ivec2(1, -1));

                total += is_alive(id + ivec2(-1, 0));
                // skip self
                total += is_alive(id + ivec2(1, 0));

                total += is_alive(id + ivec2(-1, 1));
                total += is_alive(id + ivec2(0, 1));
                total += is_alive(id + ivec2(1, 1));

                bool this_is_alive = (imageLoad(src, id_abs).x > 0);
                bool this_stays_alive = false;
                if(this_is_alive)
                {
                    if(total == 2 || total == 3) this_stays_alive = true;
                } else {
                    if(total == 3) this_stays_alive = true;
                }

                imageStore(dest, id_abs, uvec4(this_stays_alive ? 1 : 0));
            }
        ",
    }
}
