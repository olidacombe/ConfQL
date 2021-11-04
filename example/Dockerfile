FROM olidacombe/confql-base:v0.2.0 as builder
ENV DATA_ROOT=/data

# Build application
COPY schema.gql ./
RUN touch src/main.rs
RUN cargo build --release --bin=confql

#FROM debian:buster-slim AS runtime
#WORKDIR app
#COPY --from=builder /app/target/release/confql /usr/local/bin
#ENTRYPOINT ["/usr/local/bin/confql"]
ENTRYPOINT ["/app/target/release/confql"]
