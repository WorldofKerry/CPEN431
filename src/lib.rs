mod protos;
pub mod protocol;
pub mod application;
pub mod server;
pub mod kv_store;

fn timeit<F: FnMut() -> T, T>(name: &str, mut f: F) -> T {
    let start = std::time::Instant::now();
    let result = f();
    println!("{name} took {:?} ms", std::time::Instant::now().duration_since(start));
    result
  }