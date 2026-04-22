# earcut-rs

[![CI](https://github.com/ciscorn/earcut-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/ciscorn/earcut-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/ciscorn/earcut-rs/graph/badge.svg?token=thKlQiVjLc)](https://codecov.io/gh/ciscorn/earcut-rs)
[![Crates.io Version](https://img.shields.io/crates/v/earcut)](https://crates.io/crates/earcut)

A Rust port of the [mapbox/earcut](https://github.com/mapbox/earcut) polygon triangulation library.

- Based on the latest earcut 3.0.2 release.
- Designed to avoid unnecessary memory allocations. Internal buffers and the output index vector can be reused across multiple triangulations.
- Also provides `earcut::int::EarcutI32` for integer coordinates with exact integer predicates, but it can be slower than the float-based `Earcut` on modern CPUs.
- An additional module, `utils3d`, can project 3D coplanar polygons onto a 2D plane before triangulation.
- License: ISC

<p align="center">
<img src="./docs/image.png" width="300">
</p>


## Benchmarks

Measured on a MacBook Pro (M1 Pro).

| Polygon      | earcut.hpp  | earcut-rs (0.4.9) | earcutr (0.4.3) |
|--------------|------------:|------------------:|----------------:|
| bad_hole     |   3.55 µs/i |        2.650 µs/i |      4.415 µs/i |          
| building     |    351 ns/i |          155 ns/i |        604 ns/i |
| degenerate   |    154 ns/i |           39 ns/i |        206 ns/i |
| dude         |   5.25 µs/i |        4.535 µs/i |      8.096 µs/i |
| empty_square |    202 ns/i |           66 ns/i |        331 ns/i |
| water        |    423 µs/i |        400.9 µs/i |      801.3 µs/i |
| water2       |    338 µs/i |        291.4 µs/i |      450.3 µs/i |
| water3       |   13.6 µs/i |        13.03 µs/i |      23.46 µs/i |
| water3b      |   1.28 µs/i |        1.057 µs/i |      2.165 µs/i |
| water4       |   89.1 µs/i |        74.78 µs/i |      154.1 µs/i |
| water_huge   |  7.240 ms/i |        7.456 ms/i |      10.90 ms/i |
| water_huge2  |  15.86 ms/i |        16.26 ms/i |      22.35 ms/i |

Note: earcut.hpp and earcutr have not fully caught up with the latest mapbox/earcut.
