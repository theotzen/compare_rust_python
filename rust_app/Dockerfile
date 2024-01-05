FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./Rocket.toml ./Rocket.toml
COPY ./src ./src
# Prepare the recipe file that will be used to cache dependencies
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
# Copy the generated recipe.json from the planner stage
COPY --from=planner /app/recipe.json recipe.json
# Build the dependencies using the recipe file, this will cache the dependencies
RUN cargo chef cook --release --recipe-path recipe.json
# Copy the entire workspace
COPY . .
# Build the application
RUN cargo build --release

FROM debian:stable-slim AS runtime
WORKDIR /app
# Install ca-certificates
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
# Copy the binary from the builder stage to the runtime stage
COPY --from=builder /app/target/release/compare_configs /usr/local/bin/
# Copy the Rocket.toml configuration file
COPY --from=builder /app/Rocket.toml ./
# Set the entrypoint to the binary
ENTRYPOINT ["/usr/local/bin/compare_configs"]