use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod window;
mod camera;
mod math;
mod settings;
mod quad;
mod shader_program;
mod scene;
mod chunk;
mod world;
mod util;

fn main() {
    let mut app = window::VoxelEngine::new();
    app.init_gl();
    app.run();
    println!();
}

fn _test() {
    let a = std::sync::Arc::new(std::sync::Barrier::new(2));
    let a_copy = a.clone();
    rayon::spawn(move|| {
        (0..15).into_par_iter()
            .for_each(|x| {std::thread::sleep(std::time::Duration::from_secs(10));println!("{:?}", x);});
        a_copy.wait();
    });

    (0..1_000).for_each(|x| println!("main thread"));
    a.wait();
}
