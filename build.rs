extern crate gcc;

fn main() {
    gcc::Build::new()
                .file("src/accumulate.c")
                .flag("-march=native")
                .flag("-std=c99")
                .compile("libaccumulate.a");
}
