#![allow(panics)]
// This should not be flagged
// #[cfg(feature = "runtime-benchmarks")]

// This should
#[cfg(feature = "runtime-benchmarks")]
fn a() {

}

// This should
#[cfg(any(feature = "runtime-benchmarks", other_key = "other-value"))]
fn b() {

}

// This should
#[cfg(any(yes = "yes", feature = "runtime-benchmarks"))]
fn c() {

}

// This should not
#[cfg(any(feature = "runtime-benchmarks", test))]
fn d() {

}

// This should not
#[cfg(any(test, feature = "runtime-benchmarks"))]
fn e() {

}

// This should
#[cfg(any(feature="runtime-benchmarks"))]
fn f() {

}

fn main() {

}