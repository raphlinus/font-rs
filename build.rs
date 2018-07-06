#[cfg(target_arch = "x86")]
extern crate gcc;

fn main() {
    #[cfg(target_arch = "x86")]
    gcc::Build::new()
        .file("src/accumulate.c")
        .flag("-march=native")
        .flag("-std=c99")
        .compile("libaccumulate.a");
}
