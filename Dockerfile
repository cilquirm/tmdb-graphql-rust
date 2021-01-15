# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.49-alpine as cargo-build

RUN apk update && apk add gcc musl-dev openssl-dev

RUN rustup target add x86_64-unknown-linux-musl

RUN rustup toolchain install nightly && rustup default nightly

WORKDIR /usr/src/tmdb-graphql-rust

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/tmdb-graphql-rust*

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

COPY --from=cargo-build /usr/src/tmdb-graphql-rust/target/x86_64-unknown-linux-musl/release/tmdb-graphql-rust /usr/local/bin/tmdb-graphql-rust

CMD ["/usr/local/bin/tmdb-graphql-rust"]