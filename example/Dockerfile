FROM rust
ENV DATA_ROOT=/data

WORKDIR /app
COPY . .

RUN cargo build --release --bin=confql

ENTRYPOINT ["/app/target/release/confql"]
