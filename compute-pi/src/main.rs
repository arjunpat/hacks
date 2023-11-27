// #![feature(neon)]
use std::arch::aarch64::*;

use rand::prelude::*;
use rayon::prelude::*;

fn compute_pi_rayon() {
    // let xs: Vec<[f32; 4]> = Vec::with_capacity(capacity);
    // let ys: Vec<[f32; 4]> = Vec::with_capacity(capacity);
    // let xs: [[f32; 4]; 1024] = [[0.0; 4]; 1024];
    // let ys: [[f32; 4]; 1024] = [[0.0; 4]; 1024];

    // xs.par_iter().for_each(op)
    let threads = 16;
    let iters_to_run = (2 as usize).pow(32);
    let mut counts: Vec<u32> = vec![0; threads];
    counts.par_iter_mut().for_each(|count_ptr| {
        let mut arr_a: [f32; 4] = [0.0; 4];
        let mut arr_b: [f32; 4] = [0.0; 4];
        let mut rng = rand::thread_rng();

        let mut count = 0;

        assert!(iters_to_run % 4 == 0);

        for i in 0..(iters_to_run / 4) {
            if i % 100000000 == 0 {
                println!(
                    "Thread {}: {}% done",
                    rayon::current_thread_index().unwrap(),
                    (i as f64 / iters_to_run as f64) * 400.0
                );
            }

            arr_a = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            arr_b = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            let mut sum: u32 = 0;
            unsafe {
                let a = vld1q_f32(&arr_a as *const f32);
                let b = vld1q_f32(&arr_b as *const f32);

                let rad = vmlaq_f32(vmulq_f32(a, a), b, b);

                let less_than_or_equal = vcleq_f32(rad, vdupq_n_f32(1.0));
                let result_as_one_zero = vshrq_n_u32(less_than_or_equal, 31);

                let pairwise_sum = vpadd_u32(
                    vget_low_u32(result_as_one_zero),
                    vget_high_u32(result_as_one_zero),
                );
                sum = vget_lane_u32(pairwise_sum, 0) + vget_lane_u32(pairwise_sum, 1);
            }

            count += sum;
        }

        *count_ptr = count;
    });

    let approx = counts
        .iter()
        .map(|e| 4.0 * (*e as f64) / (threads * iters_to_run) as f64)
        .sum::<f64>();

    println!("PI ({:?}): {:?}", threads * iters_to_run, approx);
}

fn compute_pi_simd() {
    let mut total = 1;
    let mut count = 1;
    let mut rng = rand::thread_rng();
    let mut arr_a: [f32; 4] = [0.0; 4];
    let mut arr_b: [f32; 4] = [0.0; 4];
    unsafe {
        while total < 1e9 as u32 {
            arr_a = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            arr_b = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];

            let a = vld1q_f32(&arr_a as *const f32);
            let b = vld1q_f32(&arr_b as *const f32);

            let rad = vmlaq_f32(vmulq_f32(a, a), b, b);

            let less_than_or_equal = vcleq_f32(rad, vdupq_n_f32(1.0));
            let result_as_one_zero = vshrq_n_u32(less_than_or_equal, 31);

            let pairwise_sum = vpadd_u32(
                vget_low_u32(result_as_one_zero),
                vget_high_u32(result_as_one_zero),
            );
            let sum = vget_lane_u32(pairwise_sum, 0) + vget_lane_u32(pairwise_sum, 1);

            assert!(sum <= 4);
            count += sum;
            total += 4;

            if total % 50000000 < 4 {
                println!(
                    "PI ({:?}): {:?}",
                    total,
                    (count as f64 / total as f64) * 4.0
                );
            }
        }
    }
}

fn compute_pi() {
    let mut total = 1;
    let mut count = 1;
    let mut xs: Vec<f32> = vec![0.0; 33554432];
    let mut ys: Vec<f32> = vec![0.0; 33554432];
    let mut rng = rand::thread_rng();

    while total < 1e9 as u32 {
        for i in 0..xs.len() {
            xs[i] = rng.gen();
            ys[i] = rng.gen();
        }

        let mut inside = 0;
        for i in 0..xs.len() {
            if xs[i].powi(2) + ys[i].powi(2) <= 1.0 {
                inside += 1;
            }
        }

        total += xs.len() as u32;
        count += inside;

        println!(
            "PI ({:?}): {:?}",
            total,
            (count as f64 / total as f64) * 4.0
        );
    }
}

fn compute_pi_b() {
    let mut total = 1;
    let mut count = 1;
    let mut rng = rand::thread_rng();

    while total < 1e9 as u32 {
        let loop_size = 50000000;
        for _ in 0..loop_size {
            let x: f32 = rng.gen();
            let y: f32 = rng.gen();

            if x.powi(2) + y.powi(2) <= 1.0 {
                count += 1;
            }
        }

        total += loop_size;

        println!(
            "PI ({:?}): {:?}",
            total,
            (count as f64 / total as f64) * 4.0
        );
    }
}

fn main() {
    compute_pi_rayon();
}
