FROM rust
COPY target/debug/RustNetTest /bin/RustNetTest
EXPOSE 8888
CMD ["/bin/RustNetTest"]
