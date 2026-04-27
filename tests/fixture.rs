use earcut::int::{EarcutI32, deviation as int_deviation};
use earcut::{Earcut, deviation as float_deviation};

type Coords = Vec<Vec<[f64; 2]>>;

struct Fixture {
    data_f: Vec<[f64; 2]>,
    hole_indices: Vec<u32>,
}

fn parse_fixture(coords: &str) -> Fixture {
    let rings = serde_json::from_str::<Coords>(coords).unwrap();
    let data_f: Vec<[f64; 2]> = rings.iter().flatten().copied().collect();
    let num_rings = rings.len();
    let mut hole_indices = Vec::with_capacity(num_rings.saturating_sub(1));
    let mut sum = 0u32;
    for ring in rings.iter().take(num_rings.saturating_sub(1)) {
        sum += ring.len() as u32;
        hole_indices.push(sum);
    }

    Fixture {
        data_f,
        hole_indices,
    }
}

fn as_i32_points(data: &[[f64; 2]]) -> Option<Vec<[i32; 2]>> {
    let mut points = Vec::with_capacity(data.len());
    for &[x, y] in data {
        if x.fract() != 0.0 || y.fract() != 0.0 {
            return None;
        }
        if !(i32::MIN as f64..=i32::MAX as f64).contains(&x)
            || !(i32::MIN as f64..=i32::MAX as f64).contains(&y)
        {
            return None;
        }
        points.push([x as i32, y as i32]);
    }
    Some(points)
}

fn test_fixture(coords: &str, num_triangles: usize, expected_deviation: f64) {
    let fixture = parse_fixture(coords);

    let mut triangles = vec![];
    let mut earcut = Earcut::new();
    earcut.earcut(
        fixture.data_f.iter().copied(),
        &fixture.hole_indices,
        &mut triangles,
    );

    assert_eq!(triangles.len(), num_triangles * 3);
    let f_deviation = if triangles.is_empty() {
        0.0
    } else {
        float_deviation(
            fixture.data_f.iter().copied(),
            &fixture.hole_indices,
            &triangles,
        )
    };
    assert!(f_deviation <= expected_deviation);

    check_int_fixture_if_applicable(&fixture, triangles.len(), f_deviation);
}

fn check_int_fixture_if_applicable(fixture: &Fixture, f_triangle_indices: usize, f_deviation: f64) {
    let Some(data_i32) = as_i32_points(&fixture.data_f) else {
        return;
    };

    let mut i32_triangles = vec![];
    EarcutI32::new().earcut(
        data_i32.iter().copied(),
        &fixture.hole_indices,
        &mut i32_triangles,
    );

    assert_eq!(
        i32_triangles.len(),
        f_triangle_indices,
        "int index count differs from f64 reference"
    );

    let i_abs_dev = int_deviation(
        data_i32.iter().copied(),
        &fixture.hole_indices,
        &i32_triangles,
    );
    let polygon_area = polygon_area2(&data_i32, &fixture.hole_indices);
    if polygon_area == 0 {
        assert_eq!(i_abs_dev, 0, "int deviation was non-zero for zero area");
    } else {
        let i_dev = i_abs_dev as f64 / polygon_area as f64;
        assert!(
            i_dev <= f_deviation + 1e-12,
            "int deviation {i_dev} exceeded f64 deviation {f_deviation}"
        );
    }
}

/// Returns twice the signed area of a polygon ring (shoelace, doubled).
fn signed_area(data: &[[i32; 2]], start: usize, end: usize) -> i64 {
    let mut area = 0i64;
    let mut j = end - 1;
    for i in start..end {
        area += ((data[j][0] as i64) - (data[i][0] as i64))
            * ((data[j][1] as i64) + (data[i][1] as i64));
        j = i;
    }
    area
}

/// Returns twice the polygon area (outer minus holes); matches the scaling of
/// the value returned by `int::deviation`.
fn polygon_area2(data: &[[i32; 2]], hole_indices: &[u32]) -> i64 {
    if data.len() < 3 {
        return 0;
    }
    let outer_len = hole_indices.first().copied().unwrap_or(data.len() as u32) as usize;
    let mut area = signed_area(data, 0, outer_len).abs();
    for (i, &start) in hole_indices.iter().enumerate() {
        let start = start as usize;
        let end = if i + 1 < hole_indices.len() {
            hole_indices[i + 1] as usize
        } else {
            data.len()
        };
        if end - start >= 3 {
            area -= signed_area(data, start, end).abs();
        }
    }
    area
}

#[test]
fn fixture_building() {
    test_fixture(include_str!("fixtures/building.json"), 13, 0.0);
}

#[test]
fn fixture_dude() {
    test_fixture(include_str!("fixtures/dude.json"), 106, 2e-15);
}

#[test]
fn fixture_water1() {
    test_fixture(include_str!("fixtures/water.json"), 2482, 0.0008);
}

#[test]
fn fixture_water2() {
    test_fixture(include_str!("fixtures/water2.json"), 1212, 0.0);
}

#[test]
fn fixture_water3() {
    test_fixture(include_str!("fixtures/water3.json"), 197, 0.0);
}

#[test]
fn fixture_water3b() {
    test_fixture(include_str!("fixtures/water3b.json"), 25, 0.0);
}

#[test]
fn fixture_water4() {
    test_fixture(include_str!("fixtures/water4.json"), 705, 0.0);
}

#[test]
fn fixture_water_huge1() {
    test_fixture(include_str!("fixtures/water-huge.json"), 5176, 0.0011);
}

#[test]
fn fixture_water_huge2() {
    test_fixture(include_str!("fixtures/water-huge2.json"), 4462, 0.004);
}

#[test]
fn fixture_degenerate() {
    test_fixture(include_str!("fixtures/degenerate.json"), 0, 0.0);
}

#[test]
fn fixture_bad_hole() {
    test_fixture(include_str!("fixtures/bad-hole.json"), 42, 0.019);
}

#[test]
fn fixture_empty_square() {
    test_fixture(include_str!("fixtures/empty-square.json"), 0, 0.0);
}

#[test]
fn fixture_issue16() {
    test_fixture(include_str!("fixtures/issue16.json"), 12, 4e-16);
}

#[test]
fn fixture_issue17() {
    test_fixture(include_str!("fixtures/issue17.json"), 11, 2e-16);
}

#[test]
fn fixture_steiner() {
    test_fixture(include_str!("fixtures/steiner.json"), 9, 0.0);
}

#[test]
fn fixture_issue29() {
    test_fixture(include_str!("fixtures/issue29.json"), 40, 2e-15);
}

#[test]
fn fixture_issue34() {
    test_fixture(include_str!("fixtures/issue34.json"), 139, 0.0);
}

#[test]
fn fixture_issue35() {
    test_fixture(include_str!("fixtures/issue35.json"), 844, 0.0);
}

#[test]
fn fixture_self_touching() {
    test_fixture(include_str!("fixtures/self-touching.json"), 124, 2e-13);
}

#[test]
fn fixture_outside_ring() {
    test_fixture(include_str!("fixtures/outside-ring.json"), 64, 0.0);
}

#[test]
fn fixture_simplified_us_border() {
    test_fixture(include_str!("fixtures/simplified-us-border.json"), 120, 0.0);
}

#[test]
fn fixture_touching_holes() {
    test_fixture(include_str!("fixtures/touching-holes.json"), 57, 0.0);
}

#[test]
fn fixture_touching_holes2() {
    test_fixture(include_str!("fixtures/touching-holes2.json"), 10, 0.0);
}

#[test]
fn fixture_touching_holes3() {
    test_fixture(include_str!("fixtures/touching-holes3.json"), 82, 0.0);
}

#[test]
fn fixture_touching_holes4() {
    test_fixture(include_str!("fixtures/touching-holes4.json"), 55, 0.0);
}

#[test]
fn fixture_touching_holes5() {
    test_fixture(include_str!("fixtures/touching-holes5.json"), 133, 0.0);
}

#[test]
fn fixture_touching_holes6() {
    test_fixture(include_str!("fixtures/touching-holes6.json"), 3098, 0.0);
}

#[test]
fn fixture_hole_touching_outer() {
    test_fixture(include_str!("fixtures/hole-touching-outer.json"), 77, 0.0);
}

#[test]
fn fixture_hilbert() {
    test_fixture(include_str!("fixtures/hilbert.json"), 1024, 0.0);
}

#[test]
fn fixture_issue45() {
    test_fixture(include_str!("fixtures/issue45.json"), 10, 0.0);
}

#[test]
fn fixture_eberly_3() {
    test_fixture(include_str!("fixtures/eberly-3.json"), 73, 0.0);
}

#[test]
fn fixture_eberly_6() {
    test_fixture(include_str!("fixtures/eberly-6.json"), 1429, 2e-14);
}

#[test]
fn fixture_issue52() {
    test_fixture(include_str!("fixtures/issue52.json"), 109, 0.0);
}

#[test]
fn fixture_shared_points() {
    test_fixture(include_str!("fixtures/shared-points.json"), 4, 0.0);
}

#[test]
fn fixture_bad_diagonals() {
    test_fixture(include_str!("fixtures/bad-diagonals.json"), 7, 0.0);
}

#[test]
fn fixture_issue83() {
    test_fixture(include_str!("fixtures/issue83.json"), 0, 0.0);
}

#[test]
fn fixture_issue107() {
    test_fixture(include_str!("fixtures/issue107.json"), 0, 0.0);
}

#[test]
fn fixture_issue111() {
    test_fixture(include_str!("fixtures/issue111.json"), 18, 0.0);
}

#[test]
fn fixture_collinear_boxy() {
    test_fixture(include_str!("fixtures/boxy.json"), 58, 0.0);
}

#[test]
fn fixture_collinear_diagonal() {
    test_fixture(include_str!("fixtures/collinear-diagonal.json"), 14, 0.0);
}

#[test]
fn fixture_issue119() {
    test_fixture(include_str!("fixtures/issue119.json"), 18, 0.0);
}

#[test]
fn fixture_hourglass() {
    test_fixture(include_str!("fixtures/hourglass.json"), 2, 0.0);
}

#[test]
fn fixture_touching2() {
    test_fixture(include_str!("fixtures/touching2.json"), 8, 0.0);
}

#[test]
fn fixture_touching3() {
    test_fixture(include_str!("fixtures/touching3.json"), 15, 0.0);
}

#[test]
fn fixture_touching4() {
    test_fixture(include_str!("fixtures/touching4.json"), 19, 0.0);
}

#[test]
fn fixture_rain() {
    test_fixture(include_str!("fixtures/rain.json"), 2681, 0.0);
}

#[test]
fn fixture_issue131() {
    test_fixture(include_str!("fixtures/issue131.json"), 12, 0.0);
}

#[test]
fn fixture_infinite_loop_jhl() {
    test_fixture(include_str!("fixtures/infinite-loop-jhl.json"), 0, 0.0);
}

#[test]
fn fixture_filtered_bridge_jhl() {
    test_fixture(include_str!("fixtures/filtered-bridge-jhl.json"), 25, 0.0);
}

#[test]
fn fixture_issue149() {
    test_fixture(include_str!("fixtures/issue149.json"), 2, 0.0);
}

#[test]
fn fixture_issue142() {
    test_fixture(include_str!("fixtures/issue142.json"), 4, 0.13);
}

#[test]
fn fixture_issue186() {
    test_fixture(include_str!("fixtures/issue186.json"), 41, 0.0);
}
