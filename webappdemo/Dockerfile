FROM rust:latest as build

WORKDIR /app

RUN cargo install cargo-web

# Build dependencies first without source, so that source changes don't cause dependencies to be recompiled.
COPY Cargo.toml /app/Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"Temporary main used for building deps.\")}" > src/main.rs
RUN cargo-web build --release
RUN rm -f target/wasm32-unknown-unknown/release/deps/webappdemo*

# Now copy source and rebuild.
COPY src/ /app/src/
COPY static /app/static/
RUN cargo-web build --release
RUN cargo-web deploy --release


FROM nginx:alpine
COPY --from=build /app/target/deploy/ /usr/share/nginx/html/
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
