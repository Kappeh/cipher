# Use the official Rust image as a base
FROM rust:1.84-bullseye

# Set the working directory to /app
WORKDIR /app

# Copy files used to build application into working directory
COPY ./Cargo.toml ./
COPY ./Cargo.lock ./

COPY ./cipher_core ./cipher_core
COPY ./cipher_database ./cipher_database
COPY ./cipher_discord_bot ./cipher_discord_bot

# Build the application using Cargo
RUN cargo build --release

# Copy the built application into the working directory
# COPY target/release/cipher_discord_bot /app/cipher_discord_bot

# Run the application when the container starts
CMD ./target/release/cipher_discord_bot start
