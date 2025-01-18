use binary_raster::BinaryRaster;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{rngs::ThreadRng, Rng};
const MAIN_W: usize = 800;
const MAIN_H: usize = 400;

fn random_raster(rng: &mut ThreadRng, width: usize, height: usize, zero_to_one_ratio: u8) -> BinaryRaster {
    let pixels = (0..width*height).map(|_| 1-rng.gen_range(0..=zero_to_one_ratio).min(1)).collect::<Vec<_>>();
    BinaryRaster::from_raster(&pixels, width)
}

fn bench_collision_at(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let main_raster = random_raster(&mut rng, MAIN_W, MAIN_H, 10);
    let other_raster = random_raster(&mut rng, 100, 20, 2);
    let pos = (MAIN_W/2, MAIN_H/2);
    c.bench_function(&format!("collision_on_{}x{}", MAIN_W, MAIN_H), |b| b.iter(|| {
        main_raster.collision_check_at(black_box(&other_raster), black_box(pos));
    }));
}

criterion_group!(
    mesh, 
    bench_collision_at, 
);
criterion_main!(mesh);