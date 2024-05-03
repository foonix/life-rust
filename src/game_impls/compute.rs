use std::sync::Arc;

use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
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
    game_state: Subbuffer<[u32]>,
    cs: Arc<ComputePipeline>,
    bounds_buffer: Subbuffer<[u32]>,
}

impl GameState {
    pub fn from_random(context: Arc<VulkanContext>, size: usize) -> GameState {
        let game_state =
            context.compute_buffer_from_iter((0..size * size).map(|_| rand::random::<u32>() & 1));

        let bounds_buffer =
            context.uniform_buffer_from_iter([size as u32, size as u32].into_iter());

        let compute_pipeline = Self::create_pipeline(context.clone());

        GameState {
            size: (size, size),
            context: context.clone(),
            game_state,
            cs: compute_pipeline,
            bounds_buffer,
        }
    }

    fn create_pipeline(context: Arc<VulkanContext>) -> Arc<ComputePipeline> {
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

        ComputePipeline::new(
            context.device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .expect("failed to create compute pipeline")
    }
}

impl Gol for GameState {
    fn from_slice(size: usize, vec: &[bool]) -> Self
    where
        Self: Sized,
    {
        let context = Arc::new(VulkanContext::try_create().unwrap());

        let game_state =
            context.compute_buffer_from_iter(vec.iter().map(|b| if *b { 1 } else { 0 }));

        let bounds_buffer =
            context.uniform_buffer_from_iter([size as u32, size as u32].into_iter());

        let compute_pipeline = Self::create_pipeline(context.clone());

        GameState {
            size: (size, size),
            context: context.clone(),
            game_state,
            cs: compute_pipeline,
            bounds_buffer,
        }
    }

    fn to_vec(&self) -> Vec<bool> {
        let foo = self.game_state.read().unwrap();
        Vec::from_iter(foo.iter().map(|x| *x > 0u32))
    }

    fn to_next(&self) -> Box<dyn Gol> {
        let next_state = self
            .context
            .compute_buffer_uninit(self.size.0 * self.size.1);

        let pipeline_layout = self.cs.layout();

        let descriptor_set_layouts = pipeline_layout.set_layouts();
        let descriptor_set_layout_index = 0;
        let descriptor_set_layout = descriptor_set_layouts
            .get(descriptor_set_layout_index)
            .unwrap();
        let descriptor_set = PersistentDescriptorSet::new(
            &self.context.descriptor_set_allocator,
            descriptor_set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, self.bounds_buffer.clone()),
                WriteDescriptorSet::buffer(1, self.game_state.clone()),
                WriteDescriptorSet::buffer(2, next_state.clone()),
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

        let work_group_counts = [self.size.0 as u32 / 8 + 1, self.size.1 as u32 / 8+1, 1];

        command_buffer_builder
            .bind_pipeline_compute(self.cs.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.cs.layout().clone(),
                descriptor_set_layout_index as u32,
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
            cs: self.cs.clone(),
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

            layout(set = 0, binding = 1) buffer DataIn {
                uint src[];
            };

            layout(set = 0, binding = 2) buffer DataOut {
                uint dest[];
            };

            uint coords_to_idx(uvec2 coords)
            {
                uvec2 wrapped_coords = coords % params.game_size;
                uint ret = wrapped_coords.y * params.game_size.y + wrapped_coords.x;
                return ret;
            }

            uint is_alive(uvec2 coords)
            {
                uint index_abs = coords_to_idx(coords);
                
                return src[index_abs];
            }

            void main() {
                if(gl_GlobalInvocationID.x >= params.game_size.x || gl_GlobalInvocationID.y >= params.game_size.y)
                {
                    return;
                }

                uvec2 id = params.game_size + gl_GlobalInvocationID.xy;

                uint total = 0;
                total += is_alive(id + uvec2(-1, -1));
                total += is_alive(id + uvec2(0, -1));
                total += is_alive(id + uvec2(1, -1));

                total += is_alive(id + uvec2(-1, 0));
                // skip self
                total += is_alive(id + uvec2(1, 0));

                total += is_alive(id + uvec2(-1, 1));
                total += is_alive(id + uvec2(0, 1));
                total += is_alive(id + uvec2(1, 1));

                uint id_abs = coords_to_idx(id);
                bool this_is_alive = (src[id_abs] > 0);
                bool this_stays_alive = false;
                if(this_is_alive)
                {
                    if(total == 2 || total == 3) this_stays_alive = true;
                } else {
                    if(total == 3) this_stays_alive = true;
                }

                dest[id_abs] = this_stays_alive ? 1 : 0;
            }
        ",
    }
}
