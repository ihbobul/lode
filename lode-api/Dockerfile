FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build --release -p lode-api

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
&& apt-get install -y --no-install-recommends openssl ca-certificates \
# Clean up
&& apt-get autoremove -y \
&& apt-get clean -y \
&& rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lode-api .

COPY ./lode-api/configuration ./configuration

ENV APP_ENVIRONMENT=production

EXPOSE 8081

CMD ["./lode-api"] 