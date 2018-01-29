extern crate gcc;

#[cfg(not(target_env = "msvc"))]
fn main() {
    gcc::Build::new()
                .file("src/accumulate.c")
                .flag("-march=native")
                .flag("-std=c99")
                .compile("libaccumulate.a");
}

#[cfg(target_env = "msvc")]
fn main() {
    gcc::Config::new()
                .file("src/accumulate.c")
                // Maximum optimizations
                .flag("/Ox")
                .compile("libaccumulate.a");
}
