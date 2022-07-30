FROM rust
COPY target/debug/rust_net /bin/RustNetTest
EXPOSE 8888
CMD ["/bin/RustNetTest"]
