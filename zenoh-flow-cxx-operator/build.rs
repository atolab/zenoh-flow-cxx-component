fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("cpp/src/cxx_fizz_operator.cpp")
        .flag_if_supported("-std=c++14")
        .compile("cxx_fizz_operator");
}
