# earcut (Rust)

[![CI](https://github.com/georust/earcut/actions/workflows/ci.yml/badge.svg)](https://github.com/georust/earcut/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/georust/earcut/graph/badge.svg)](https://codecov.io/gh/georust/earcut)
[![Crates.io Version](https://img.shields.io/crates/v/earcut)](https://crates.io/crates/earcut)

A Rust port of the [mapbox/earcut](https://github.com/mapbox/earcut) polygon triangulation library.

- Tracks the triangulation behavior and optional Delaunay refinement from the latest earcut 3.2.3.
- Designed to avoid unnecessary memory allocations. Internal buffers and the output index vector can be reused across multiple triangulations.
- Also provides `earcut::int::EarcutI32` for integer coordinates with exact integer predicates, but it can be slower than the float-based `Earcut` on modern CPUs.
- An additional helper, `utils3d::project3d_to_2d`, projects coplanar 3D polygons onto a 2D plane for use with earcut.

<p align="center">
<img src="./docs/image.png" width="300">
</p>


## Benchmarks

Time per iteration (smaller is better). Measured on a MacBook Pro (M1 Pro).

| Polygon      | earcut.hpp (C++) | earcut (Rust) |
|--------------|-----------------:|--------------:|
| bad_hole     |        2.27 µs/i |    2.337 µs/i |
| building     |         294 ns/i |    167.7 ns/i |
| degenerate   |         139 ns/i |    42.63 ns/i |
| dude         |        4.46 µs/i |    4.275 µs/i |
| empty_square |         271 ns/i |    83.22 ns/i |
| water        |         284 µs/i |    236.2 µs/i |
| water2       |         192 µs/i |    155.7 µs/i |
| water3       |        12.8 µs/i |    13.78 µs/i |
| water3b      |        1.26 µs/i |    1.060 µs/i |
| water4       |        53.6 µs/i |    58.69 µs/i |
| water_huge   |       1.399 ms/i |    1.298 ms/i |
| water_huge2  |       2.694 ms/i |    2.511 ms/i |
| water_huge3  |       21.84 ms/i |    19.50 ms/i |
| MVT corpus   |         250 ms/i |      248 ms/i |

## Demo

A simple egui-based visualizer for inspecting how earcut works.

```bash
cargo run --example visualizer
```

<p align="center">
<img src="./docs/visualizer.png" width="auto">
</p>

## License

Licensed under either the MIT License ([LICENSE-MIT](./LICENSE-MIT)) or the Apache License 2.0 ([LICENSE-APACHE](./LICENSE-APACHE)) at your option.

This project contains portions derived from [mapbox/earcut](https://github.com/mapbox/earcut), originally distributed under the ISC License ([LICENSE-ISC](./LICENSE-ISC)).
