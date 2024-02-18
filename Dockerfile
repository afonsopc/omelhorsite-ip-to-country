# Second stage: build the actual application
FROM rust AS application-builder

WORKDIR /app

COPY . .

RUN cargo build --release

RUN mv /app/target/release/ip-to-country /compiled_binary


# Third stage: create the final image
FROM rust:slim

COPY --from=application-builder /compiled_binary /compiled_binary

CMD ["/compiled_binary"]