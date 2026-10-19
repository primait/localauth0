FROM 279066465364.dkr.ecr.eu-west-1.amazonaws.com/prima-rust:1.98.1

WORKDIR /code

ENV CARGO_HOME=/home/app/.cargo

COPY entrypoint /code/entrypoint

# Needed to have the same file owner in the container and in Linux host
USER app

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --version ^0.17 trunk

ENTRYPOINT ["./entrypoint"]
