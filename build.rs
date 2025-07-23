fn main() {
    println!("cargo:rustc-link-arg=-lc++");
    println!("cargo:rustc-link-arg=-lc++abi");
}
