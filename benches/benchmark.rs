use std::fs;

use criterion::{Criterion, criterion_group, criterion_main};

use earcut::Earcut;
use earcut::int::EarcutI32;

type Coords = Vec<Vec<[f64; 2]>>;

fn load_fixture(name: &str) -> (Vec<[f64; 2]>, Vec<u32>) {
    let s = fs::read_to_string("./tests/fixtures/".to_string() + name + ".json").unwrap();
    let expected = serde_json::from_str::<Coords>(&s).unwrap();

    let num_rings = expected.len();
    let data = expected.clone().into_iter().flatten().collect::<Vec<_>>();
    let hole_indices: Vec<_> = expected
        .iter()
        .map(|x| x.len() as u32)
        .scan(0, |sum, e| {
            *sum += e;
            Some(*sum)
        })
        .take(num_rings.saturating_sub(1))
        .collect();

    (data, hole_indices)
}

/// Load the same fixture as `i32` pairs. Returns `None` if any coordinate
/// is not integer-valued or falls outside `i32`.
fn load_fixture_i32(name: &str) -> Option<(Vec<[i32; 2]>, Vec<u32>)> {
    let (data, holes) = load_fixture(name);
    let mut out = Vec::with_capacity(data.len());
    for [x, y] in data {
        if x.fract() != 0.0 || y.fract() != 0.0 {
            return None;
        }
        if !(i32::MIN as f64..=i32::MAX as f64).contains(&x)
            || !(i32::MIN as f64..=i32::MAX as f64).contains(&y)
        {
            return None;
        }
        out.push([x as i32, y as i32]);
    }
    Some((out, holes))
}

const F64_FIXTURES: &[&str] = &[
    "bad-hole",
    "building",
    "degenerate",
    "dude",
    "empty-square",
    "water",
    "water2",
    "water3",
    "water3b",
    "water4",
    "water-huge",
    "water-huge2",
];

/// Subset of `F64_FIXTURES` that are integer-representable (every bench we
/// have today except `dude`).
const INT_FIXTURES: &[&str] = &[
    "bad-hole",
    "building",
    "degenerate",
    "empty-square",
    "water",
    "water2",
    "water3",
    "water3b",
    "water4",
    "water-huge",
    "water-huge2",
];

fn bench(c: &mut Criterion) {
    let mut earcut = Earcut::new();
    let mut triangles: Vec<u32> = Vec::new();
    for name in F64_FIXTURES {
        let (data, hole_indices) = load_fixture(name);
        c.bench_function(name, |b| {
            b.iter(|| {
                earcut.earcut(data.iter().copied(), &hole_indices, &mut triangles);
            })
        });
    }

    let mut earcut_int = EarcutI32::new();
    let mut triangles_i: Vec<u32> = Vec::new();
    for name in INT_FIXTURES {
        let (data, hole_indices) = load_fixture_i32(name).expect("fixture is integer-valued");
        c.bench_function(&format!("int/{name}"), |b| {
            b.iter(|| {
                earcut_int.earcut(data.iter().copied(), &hole_indices, &mut triangles_i);
            })
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
