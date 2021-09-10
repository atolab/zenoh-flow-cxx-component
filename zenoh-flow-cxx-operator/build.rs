fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("cpp/src/operator.cpp")
        .flag_if_supported("-std=c++14")
        .compile("cxx_operator");
}
