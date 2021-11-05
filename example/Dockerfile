FROM olidacombe/confql-example-base:0.3.0 as builder
COPY schema.gql .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin confql

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/confql /usr/local/bin/
USER myuser
ENV DATA_ROOT=/data
CMD ["/usr/local/bin/confql"]
