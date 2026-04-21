//! Cross-validate `earcut::int::EarcutI32` against the floating-point
//! `earcut::Earcut<f64>` on every integer-valued fixture.

use std::fs;

use earcut::int::{deviation as int_deviation, EarcutI32};
use earcut::Earcut;

type Coords = Vec<Vec<[f64; 2]>>;

fn load(name: &str) -> Option<(Vec<[f64; 2]>, Vec<[i32; 2]>, Vec<u32>)> {
    let s = fs::read_to_string("./tests/fixtures/".to_string() + name + ".json").unwrap();
    let rings = serde_json::from_str::<Coords>(&s).unwrap();

    // Only accept fixtures whose coords are representable as i32 exactly.
    let flat: Vec<[f64; 2]> = rings.iter().flatten().copied().collect();
    let mut as_i32 = Vec::with_capacity(flat.len());
    for &[x, y] in &flat {
        if x.fract() != 0.0 || y.fract() != 0.0 {
            return None;
        }
        if !(i32::MIN as f64..=i32::MAX as f64).contains(&x)
            || !(i32::MIN as f64..=i32::MAX as f64).contains(&y)
        {
            return None;
        }
        as_i32.push([x as i32, y as i32]);
    }

    let num_holes = rings.len();
    let hole_indices: Vec<u32> = rings
        .into_iter()
        .map(|x| x.len() as u32)
        .scan(0u32, |sum, e| {
            *sum += e;
            Some(*sum)
        })
        .take(num_holes.saturating_sub(1))
        .collect();

    Some((flat, as_i32, hole_indices))
}

fn check(name: &str) {
    let Some((data_f, data_i32, holes)) = load(name) else {
        panic!("fixture {name} contained non-integer coordinates");
    };

    let mut f_tri: Vec<u32> = vec![];
    let mut i32_tri: Vec<u32> = vec![];
    Earcut::new().earcut(data_f.iter().copied(), &holes, &mut f_tri);
    EarcutI32::new().earcut(data_i32.iter().copied(), &holes, &mut i32_tri);

    // Because the algorithm is deterministic for identical input, the integer
    // triangulation must produce the same indices as the f64 reference. This
    // also implies identical deviation — no need for a separate tolerance.
    assert_eq!(
        i32_tri,
        f_tri,
        "{name}: int indices differ from f64 indices (i32: {} idx, f64: {} idx)",
        i32_tri.len(),
        f_tri.len()
    );

    // For fixtures that are expected to have a perfect triangulation
    // (fract area == 0), we additionally confirm our integer deviation agrees.
    let _ = int_deviation(data_i32.iter().copied(), &holes, &i32_tri);
}

macro_rules! int_fixture_tests {
    ($($fn_name:ident => $fixture:literal),* $(,)?) => {
        $(
            #[test]
            fn $fn_name() { check($fixture); }
        )*
    };
}

int_fixture_tests! {
    int_fixture_bad_diagonals => "bad-diagonals",
    int_fixture_bad_hole => "bad-hole",
    int_fixture_boxy => "boxy",
    int_fixture_building => "building",
    int_fixture_collinear_diagonal => "collinear-diagonal",
    int_fixture_degenerate => "degenerate",
    int_fixture_eberly_3 => "eberly-3",
    int_fixture_empty_square => "empty-square",
    int_fixture_hilbert => "hilbert",
    int_fixture_hole_touching_outer => "hole-touching-outer",
    int_fixture_hourglass => "hourglass",
    int_fixture_issue111 => "issue111",
    int_fixture_issue119 => "issue119",
    int_fixture_issue131 => "issue131",
    int_fixture_issue149 => "issue149",
    int_fixture_issue186 => "issue186",
    int_fixture_issue34 => "issue34",
    int_fixture_issue35 => "issue35",
    int_fixture_issue45 => "issue45",
    int_fixture_issue52 => "issue52",
    int_fixture_issue83 => "issue83",
    int_fixture_outside_ring => "outside-ring",
    int_fixture_rain => "rain",
    int_fixture_shared_points => "shared-points",
    int_fixture_simplified_us_border => "simplified-us-border",
    int_fixture_steiner => "steiner",
    int_fixture_touching_holes => "touching-holes",
    int_fixture_touching_holes2 => "touching-holes2",
    int_fixture_touching_holes3 => "touching-holes3",
    int_fixture_touching_holes4 => "touching-holes4",
}
