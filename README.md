# earcut-rs

[![Test](https://github.com/ciscorn/earcut-rs/actions/workflows/Test.yml/badge.svg)](https://github.com/ciscorn/earcut-rs/actions/workflows/Test.yml)
[![codecov](https://codecov.io/gh/ciscorn/earcut-rs/graph/badge.svg?token=thKlQiVjLc)](https://codecov.io/gh/ciscorn/earcut-rs)
[![Crates.io Version](https://img.shields.io/crates/v/earcut)](https://crates.io/crates/earcut)

A Rust port of the [mapbox/earcut](https://github.com/mapbox/earcut) polygon triangulation library.

- Based on the latest earcut 3.0.2 release.
- Designed to avoid unnecessary memory allocations. Internal buffers and the output index vector can be reused across multiple triangulations.
- An additional module, `utils3d`, can project 3D coplanar polygons onto a 2D plane before triangulation.
- License: ISC

<p align="center">
<img src="./docs/image.png" width="300">
</p>


## Benchmarks

Measured on a MacBook Pro (M1 Pro).

| Polygon      | earcut.hpp  | earcut-rs (0.4.7) | earcutr (0.4.3) |
|--------------|------------:|------------------:|----------------:|
| bad_hole     |   3.55 µs/i |        2.664 µs/i |      4.415 µs/i |          
| building     |    351 ns/i |          158 ns/i |        604 ns/i |
| degenerate   |    154 ns/i |           38 ns/i |        206 ns/i |
| dude         |   5.25 µs/i |        4.408 µs/i |      8.096 µs/i |
| empty_square |    202 ns/i |           63 ns/i |        331 ns/i |
| water        |    423 µs/i |        394.8 µs/i |      801.3 µs/i |
| water2       |    338 µs/i |        270.6 µs/i |      450.3 µs/i |
| water3       |   13.6 µs/i |        12.66 µs/i |      23.46 µs/i |
| water3b      |   1.28 µs/i |        1.067 µs/i |      2.165 µs/i |
| water4       |   89.1 µs/i |        72.26 µs/i |      154.1 µs/i |
| water_huge   |  7.240 ms/i |        7.459 ms/i |      10.90 ms/i |
| water_huge2  |  15.86 ms/i |        16.57 ms/i |      22.35 ms/i |

Note: earcutr 0.4.3 is not based on the latest mapbox/earcut release.
