use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};

use earcut::int::{EarcutI32, deviation};

const LIMIT: i32 = 1 << 19;
const SEED: u64 = 0xa11c_e5ed_5eed_f00d;

fn triangle_area2(a: [i32; 2], b: [i32; 2], c: [i32; 2]) -> i64 {
    (i64::from(b[0]) - i64::from(a[0])) * (i64::from(c[1]) - i64::from(a[1]))
        - (i64::from(b[1]) - i64::from(a[1])) * (i64::from(c[0]) - i64::from(a[0]))
}

fn assert_exact_triangulation(data: &[[i32; 2]], holes: &[u32], context: &str) {
    let mut triangles = Vec::new();
    EarcutI32::new().earcut(data.iter().copied(), holes, &mut triangles);

    assert_eq!(triangles.len() % 3, 0, "partial triangle: {context}");
    for (triangle_index, triangle) in triangles.chunks_exact(3).enumerate() {
        let [a, b, c] = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        assert!(
            a < data.len() && b < data.len() && c < data.len(),
            "out-of-range triangle {triangle_index}: {context}"
        );
        assert_ne!(
            triangle_area2(data[a], data[b], data[c]),
            0,
            "zero-area triangle {triangle_index}: {context}"
        );
    }
    assert_eq!(
        deviation(data.iter().copied(), holes, &triangles),
        0,
        "area mismatch: {context}"
    );
}

/// Generates an x-monotone polygon. Keeping the upper chain strictly above the
/// lower chain makes the result simple while still allowing severe concavity.
fn random_monotone_polygon(rng: &mut fastrand::Rng, points_per_chain: usize) -> Vec<[i32; 2]> {
    let mut lower = Vec::with_capacity(points_per_chain);
    let mut upper = Vec::with_capacity(points_per_chain);
    let denominator = (points_per_chain - 1) as i64;

    for i in 0..points_per_chain {
        let x = -LIMIT + ((2 * i64::from(LIMIT) * i as i64) / denominator) as i32;
        lower.push([x, -(1 + rng.i32(0..LIMIT - 1))]);
        upper.push([x, 1 + rng.i32(0..LIMIT - 1)]);
    }

    lower.extend(upper.into_iter().rev());
    lower
}

fn rotate(point: [i32; 2], quarter_turns: usize) -> [i32; 2] {
    match quarter_turns % 4 {
        0 => point,
        1 => [-point[1], point[0]],
        2 => [-point[0], -point[1]],
        3 => [point[1], -point[0]],
        _ => unreachable!(),
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic payload".to_owned())
}

#[test]
fn exact_for_random_simple_polygons_at_domain_extremes() {
    let mut rng = fastrand::Rng::with_seed(SEED);

    for iteration in 0..256 {
        // Alternates below and above the 80-vertex z-order hashing threshold.
        let points_per_chain = if iteration % 2 == 0 {
            3 + rng.usize(0..35)
        } else {
            41 + rng.usize(0..55)
        };
        let mut polygon = random_monotone_polygon(&mut rng, points_per_chain);
        let quarter_turns = iteration % 4;
        for point in &mut polygon {
            *point = rotate(*point, quarter_turns);
        }
        if rng.bool() {
            polygon.reverse();
        }

        assert_exact_triangulation(
            &polygon,
            &[],
            &format!(
                "iteration {iteration}, seed {SEED:#018x}, vertices {}, rotation {quarter_turns}",
                polygon.len()
            ),
        );
    }
}

#[test]
fn exact_with_many_tied_holes_near_domain_extremes() {
    for variant in 0..8 {
        let mut rings = vec![vec![
            [-LIMIT, -LIMIT],
            [LIMIT, -LIMIT],
            [LIMIT, LIMIT],
            [-LIMIT, LIMIT],
        ]];

        for row in 0..8 {
            for column in 0..8 {
                let x0 = -440_000 + column * 110_000;
                let y0 = -440_000 + row * 110_000;
                let mut hole = vec![
                    [x0, y0],
                    [x0 + 50_000, y0],
                    [x0 + 50_000, y0 + 50_000],
                    [x0, y0 + 50_000],
                ];
                if (row + column + variant) % 2 != 0 {
                    hole.reverse();
                }
                rings.push(hole);
            }
        }

        if variant & 1 != 0 {
            rings[0].reverse();
        }
        let quarter_turns = (variant / 2) as usize;
        let mut data = Vec::new();
        let mut holes = Vec::with_capacity(rings.len() - 1);
        for (ring_index, ring) in rings.into_iter().enumerate() {
            if ring_index != 0 {
                holes.push(data.len() as u32);
            }
            data.extend(ring.into_iter().map(|point| rotate(point, quarter_turns)));
        }

        assert_exact_triangulation(
            &data,
            &holes,
            &format!("64 holes, winding/rotation variant {variant}"),
        );
    }
}

#[test]
fn arbitrary_degenerate_inputs_remain_structurally_safe_and_deterministic() {
    let mut rng = fastrand::Rng::with_seed(SEED ^ 0x55aa_55aa_55aa_55aa);
    let mut earcut = EarcutI32::new();
    let mut first = Vec::new();
    let mut second = Vec::new();

    for iteration in 0..2_000 {
        let len = rng.usize(0..128);
        let mut data = Vec::with_capacity(len);
        for vertex in 0..len {
            let point = match vertex % 7 {
                0 if !data.is_empty() => data[data.len() - 1],
                1 => [rng.i32(-64..=64), 0],
                2 => [0, rng.i32(-64..=64)],
                _ => [rng.i32(-64..=64), rng.i32(-64..=64)],
            };
            data.push(point);
        }

        earcut.earcut(data.iter().copied(), &[] as &[u32], &mut first);
        earcut.earcut(data.iter().copied(), &[] as &[u32], &mut second);
        assert_eq!(
            first, second,
            "non-deterministic output at iteration {iteration}"
        );
        assert_eq!(
            first.len() % 3,
            0,
            "partial triangle at iteration {iteration}"
        );
        assert!(
            first.iter().all(|&index| (index as usize) < data.len()),
            "out-of-range index at iteration {iteration}"
        );
    }
}

#[test]
fn rejects_every_out_of_domain_direction_even_for_short_inputs() {
    let invalid_points = [
        [LIMIT + 1, 0],
        [-LIMIT - 1, 0],
        [0, LIMIT + 1],
        [0, -LIMIT - 1],
    ];

    for invalid in invalid_points {
        let result = catch_unwind(|| {
            let mut triangles = Vec::<u32>::new();
            EarcutI32::new().earcut([invalid], &[], &mut triangles);
        });
        let message = panic_message(result.expect_err("out-of-domain input was accepted"));
        assert!(
            message.contains("out of range"),
            "unexpected panic: {message}"
        );
    }
}

#[test]
fn rejects_malformed_holes_before_degenerate_outer_ring_early_returns() {
    let data = [[0, 0], [100, 0], [100, 100], [0, 100]];
    let malformed: &[&[u32]] = &[&[0, 5], &[0, 3, 2], &[5]];

    for &holes in malformed {
        let result = catch_unwind(|| {
            let mut triangles = Vec::new();
            EarcutI32::new().earcut(data, holes, &mut triangles);
        });
        assert!(
            result.is_err(),
            "malformed hole indices {holes:?} were accepted"
        );
    }
}

#[test]
fn rejects_u16_output_indices_instead_of_truncating_them() {
    // Duplicate padding moves every meaningful square vertex beyond u16::MAX.
    let mut data = vec![[0, 0]; usize::from(u16::MAX) + 1];
    data.extend([[100, 0], [100, 100], [0, 100]]);

    let result = catch_unwind(|| {
        let mut triangles = Vec::<u16>::new();
        EarcutI32::new().earcut(data.iter().copied(), &[], &mut triangles);
    });
    let message = panic_message(result.expect_err("u16 indices were silently truncated"));
    assert!(message.contains("u16"), "unexpected panic: {message}");
}

#[test]
fn instance_is_reusable_after_rejected_input() {
    let mut earcut = EarcutI32::new();
    let mut triangles = vec![7_u32, 8, 9];

    let result = catch_unwind(AssertUnwindSafe(|| {
        earcut.earcut([[LIMIT + 1, 0], [0, 1], [1, 0]], &[], &mut triangles);
    }));
    assert!(result.is_err());

    let square = [[0, 0], [10, 0], [10, 10], [0, 10]];
    earcut.earcut(square, &[], &mut triangles);
    assert_eq!(triangles.len(), 6);
    assert_eq!(deviation(square, &[], &triangles), 0);
}
