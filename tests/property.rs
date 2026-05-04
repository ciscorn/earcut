use earcut::int::{EarcutI32, deviation as int_deviation};
use earcut::{Earcut, Refiner, deviation};

const SEED: u64 = 0xdead_beef_cafe_babe;
const ITERATIONS: usize = 128;
const OUTER_SIZE: i32 = 1_024;
const UNIMODULAR_TRANSFORMS: [[i32; 4]; 8] = [
    [1, 0, 0, 1],
    [0, 1, -1, 0],
    [1, 3, 0, 1],
    [2, 1, 1, 1],
    [-1, 2, -1, 1],
    [-3, 5, 4, -7],
    [-1, 0, 0, 1],
    [1, -2, 1, -1],
];

fn collinear_outer_ring() -> Vec<[i32; 2]> {
    const SEGMENTS: i32 = 64;
    const STEP: i32 = OUTER_SIZE / SEGMENTS;
    let mut ring = Vec::with_capacity((SEGMENTS * 4) as usize);

    for segment in 0..SEGMENTS {
        ring.push([segment * STEP, 0]);
    }
    for segment in 0..SEGMENTS {
        ring.push([OUTER_SIZE, segment * STEP]);
    }
    for segment in (1..=SEGMENTS).rev() {
        ring.push([segment * STEP, OUTER_SIZE]);
    }
    for segment in (1..=SEGMENTS).rev() {
        ring.push([0, segment * STEP]);
    }

    ring
}

fn rectangular_hole(rng: &mut fastrand::Rng, column: i32, row: i32) -> Vec<[i32; 2]> {
    const CELL: i32 = 150;
    let x0 = 30 + column * CELL + rng.i32(0..24);
    let y0 = 30 + row * CELL + rng.i32(0..24);
    let width = 48 + rng.i32(0..36);
    let height = 48 + rng.i32(0..36);
    let x1 = x0 + width;
    let y1 = y0 + height;
    let xm = (x0 + x1) / 2;
    let ym = (y0 + y1) / 2;

    // Extra collinear points exercise filtering while holes are merged into
    // the block-indexed outer ring.
    vec![
        [x0, y0],
        [x0, ym],
        [x0, y1],
        [xm, y1],
        [x1, y1],
        [x1, ym],
        [x1, y0],
        [xm, y0],
    ]
}

fn generate_rings(rng: &mut fastrand::Rng) -> Vec<Vec<[i32; 2]>> {
    let mut rings = vec![collinear_outer_ring()];
    for row in 0..6 {
        for column in 0..6 {
            if rng.i32(0..4) != 0 {
                rings.push(rectangular_hole(rng, column, row));
            }
        }
    }
    rings
}

fn flatten(rings: &[Vec<[i32; 2]>]) -> (Vec<[i32; 2]>, Vec<u32>) {
    let vertex_count = rings.iter().map(Vec::len).sum();
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut holes = Vec::with_capacity(rings.len().saturating_sub(1));

    for (index, ring) in rings.iter().enumerate() {
        if index != 0 {
            holes.push(u32::try_from(vertices.len()).expect("test polygon is too large"));
        }
        vertices.extend_from_slice(ring);
    }
    (vertices, holes)
}

fn transform_vertices(vertices: &mut [[i32; 2]], [a, b, c, d]: [i32; 4]) {
    assert_eq!(((a as i64) * (d as i64) - (b as i64) * (c as i64)).abs(), 1);
    for [x, y] in vertices {
        (*x, *y) = (a * *x + b * *y, c * *x + d * *y);
    }
}

#[test]
fn block_index_property_regression() {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut float_earcut = Earcut::new();
    let mut int_earcut = EarcutI32::new();
    let mut refiner = Refiner::new();
    let mut float_triangles = Vec::new();
    let mut int_triangles = Vec::new();

    for iteration in 0..ITERATIONS {
        let iteration_seed = rng.get_seed();
        let rings = generate_rings(&mut rng);
        let (mut int_vertices, holes) = flatten(&rings);
        let transform = UNIMODULAR_TRANSFORMS[iteration % UNIMODULAR_TRANSFORMS.len()];
        transform_vertices(&mut int_vertices, transform);
        assert!(
            holes.len() >= 12,
            "insufficient hole stress at iteration {iteration}, seed {iteration_seed:#018x}"
        );
        let float_vertices = int_vertices
            .iter()
            .map(|&[x, y]| [f64::from(x), f64::from(y)])
            .collect::<Vec<_>>();

        float_earcut.earcut(float_vertices.iter().copied(), &holes, &mut float_triangles);
        int_earcut.earcut(int_vertices.iter().copied(), &holes, &mut int_triangles);

        let context =
            format!("iteration {iteration}, seed {iteration_seed:#018x}, transform {transform:?}");
        assert_eq!(
            deviation(float_vertices.iter().copied(), &holes, &float_triangles),
            0.0,
            "float coverage failed: {context}"
        );
        assert_eq!(
            int_deviation(int_vertices.iter().copied(), &holes, &int_triangles),
            0,
            "integer coverage failed: {context}"
        );
        assert_eq!(
            float_triangles.len(),
            int_triangles.len(),
            "float/int triangle count differs: {context}"
        );

        let triangle_count = float_triangles.len();
        refiner.refine(&mut float_triangles, &float_vertices);
        assert_eq!(
            float_triangles.len(),
            triangle_count,
            "refine changed triangle count: {context}"
        );
        assert_eq!(
            deviation(float_vertices.iter().copied(), &holes, &float_triangles),
            0.0,
            "refine changed coverage: {context}"
        );
    }
}
