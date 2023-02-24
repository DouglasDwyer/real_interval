# real_interval

[![Crates.io](https://img.shields.io/crates/v/real_interval.svg)](https://crates.io/crates/real_interval)
[![Docs.rs](https://docs.rs/real_interval/badge.svg)](https://docs.rs/real_interval)

`RealInterval` provides an `f32`-backed continuous interval type for ergonomic
interval manipulation. Scalar operations, arithmetic operations, and set operations
on intervals are all supported. The following is a simple example of how to use
intervals:

```rust
let interval = RealInterval::min_max(-1.0, 2.0);
let shifted_interval = interval + 0.5;
let expanded_interval = RealInterval::min_max(-2.0, 3.0) * interval;

assert_eq!(RealInterval::min_max(-0.5, 2.5), shifted_interval);
assert_eq!(RealInterval::min_max(-4.0, 6.0), expanded_interval);

let and_interval = interval & shifted_interval;
let or_interval = interval | shifted_interval;

assert_eq!(Some(RealInterval::min_max(-0.5, 2.0)), and_interval);
assert_eq!(RealInterval::min_max(-1.0, 2.5), or_interval);
```