FROM rust:1.67.0

WORKDIR /workdir
COPY . /workdir

RUN cargo build --release

CMD ["/workdir/target/release/rust-sns"]