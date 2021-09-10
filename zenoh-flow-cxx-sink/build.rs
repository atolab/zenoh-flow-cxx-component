fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("cpp/src/sink.cpp")
        .flag_if_supported("-std=c++14")
        .compile("cxx_sink");
}
