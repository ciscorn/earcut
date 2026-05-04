//! Streaming decoder for the upstream packed MVT polygon corpus.

const DATA: &[u8] = include_bytes!("tiles-fixture.bin");

/// Decodes every polygon in the corpus and calls `visit` without retaining the
/// complete 119,680-polygon data set in memory.
pub fn for_each_polygon(mut visit: impl FnMut(&[[f64; 2]], &[u32])) -> usize {
    let mut reader = VarintReader::new(DATA);
    let mut geometry = Vec::new();
    let mut polygons = 0;

    while !reader.is_empty() {
        let _zoom = reader.read();
        let feature_count = reader.read();

        for _ in 0..feature_count {
            let geometry_len = reader.read() as usize;
            geometry.clear();
            geometry.reserve(geometry_len);
            for _ in 0..geometry_len {
                geometry.push(reader.read());
            }
            polygons += decode_feature(&geometry, &mut visit);
        }
    }

    polygons
}

fn decode_feature(geometry: &[u32], visit: &mut impl FnMut(&[[f64; 2]], &[u32])) -> usize {
    let mut cursor = 0;
    let mut x = 0_i32;
    let mut y = 0_i32;
    let mut ring = Vec::new();
    let mut vertices = Vec::new();
    let mut holes = Vec::new();
    let mut polygons = 0;

    while cursor < geometry.len() {
        let command = geometry[cursor] & 0x7;
        let count = geometry[cursor] >> 3;
        cursor += 1;

        match command {
            1 => {
                for _ in 0..count {
                    accept_ring(&ring, &mut vertices, &mut holes, &mut polygons, visit);
                    ring.clear();
                    x = x
                        .checked_add(zigzag_decode(geometry[cursor]))
                        .expect("MVT x coordinate overflow");
                    y = y
                        .checked_add(zigzag_decode(geometry[cursor + 1]))
                        .expect("MVT y coordinate overflow");
                    cursor += 2;
                    ring.push([x, y]);
                }
            }
            2 => {
                for _ in 0..count {
                    x = x
                        .checked_add(zigzag_decode(geometry[cursor]))
                        .expect("MVT x coordinate overflow");
                    y = y
                        .checked_add(zigzag_decode(geometry[cursor + 1]))
                        .expect("MVT y coordinate overflow");
                    cursor += 2;
                    ring.push([x, y]);
                }
            }
            7 => {
                accept_ring(&ring, &mut vertices, &mut holes, &mut polygons, visit);
                ring.clear();
            }
            _ => panic!("invalid MVT command {command}"),
        }
    }

    accept_ring(&ring, &mut vertices, &mut holes, &mut polygons, visit);
    emit_polygon(&mut vertices, &mut holes, &mut polygons, visit);
    polygons
}

fn accept_ring(
    ring: &[[i32; 2]],
    vertices: &mut Vec<[f64; 2]>,
    holes: &mut Vec<u32>,
    polygons: &mut usize,
    visit: &mut impl FnMut(&[[f64; 2]], &[u32]),
) {
    if ring.len() < 3 {
        return;
    }

    let area = ring_area(ring);
    if area == 0 {
        return;
    }

    if area > 0 {
        emit_polygon(vertices, holes, polygons, visit);
    } else if vertices.is_empty() {
        return;
    } else {
        holes.push(u32::try_from(vertices.len()).expect("MVT polygon has too many vertices"));
    }

    vertices.extend(ring.iter().map(|&[x, y]| [f64::from(x), f64::from(y)]));
}

fn emit_polygon(
    vertices: &mut Vec<[f64; 2]>,
    holes: &mut Vec<u32>,
    polygons: &mut usize,
    visit: &mut impl FnMut(&[[f64; 2]], &[u32]),
) {
    if vertices.is_empty() {
        return;
    }
    visit(vertices, holes);
    *polygons += 1;
    vertices.clear();
    holes.clear();
}

fn ring_area(ring: &[[i32; 2]]) -> i64 {
    let mut sum = 0_i64;
    let mut previous = ring[ring.len() - 1];
    for &point in ring {
        sum += (i64::from(previous[0]) - i64::from(point[0]))
            * (i64::from(point[1]) + i64::from(previous[1]));
        previous = point;
    }
    sum
}

fn zigzag_decode(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}

struct VarintReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> VarintReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    fn is_empty(&self) -> bool {
        self.cursor == self.data.len()
    }

    fn read(&mut self) -> u32 {
        let mut value = 0_u32;
        let mut shift = 0;

        loop {
            let byte = *self
                .data
                .get(self.cursor)
                .expect("truncated varint in MVT corpus");
            self.cursor += 1;
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
            shift += 7;
            assert!(shift < 32, "oversized varint in MVT corpus");
        }
    }
}
