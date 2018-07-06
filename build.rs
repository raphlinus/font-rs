#[cfg(feature="sse")]
extern crate gcc;

fn main() {
    #[cfg(feature="sse")]
    gcc::Build::new()
        .file("src/accumulate.c")
        .flag("-march=native")
        .flag("-std=c99")
        .compile("libaccumulate.a");
}
