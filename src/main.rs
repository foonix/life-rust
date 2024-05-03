use game_impls::cpu;
use life_rust::game_impls::compute;
use life_rust::Gol;
use life_rust::{game_impls, VulkanContext};
use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::{thread, time};

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let mut size: usize = 16;
    if args.len() > 1 {
        let parsed = args[1].parse();
        match parsed {
            Ok(parsed_size) => size = parsed_size,
            _ => panic!(
                "Game size parameter ({}) must be a positive integer.",
                args[2]
            ),
        }
    }

    let vulkan_context: Arc<VulkanContext>;
    let mut back: Box<dyn Gol>;
    let mut front: Box<dyn Gol>;
    let vulkan_context_result = VulkanContext::try_create();
    match vulkan_context_result {
        Ok(context) => {
            vulkan_context = Arc::new(context);
            back = Box::new(compute::GameState::from_random(vulkan_context.clone(), size));
        }
        Err(e) => {
            println!("Failed to initialize Vulkan context: {:?}", e);
            println!("Falling back on CPU processing.");
            back = cpu::GameState::from_random(size);
        }
    }

    back.print();
    println!("------ <start>");
    for i in 0..32 {
        front = back.to_next();
        front.print();
        println!("------ {}", i);
        io::stdout().flush().unwrap();

        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);

        back = front;
    }
}
