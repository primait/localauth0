FROM public.ecr.aws/primaassicurazioni/rust:1.96.0

WORKDIR /code

ENV CARGO_HOME=/home/app/.cargo

COPY entrypoint /code/entrypoint

# Needed to have the same file owner in the container and in Linux host
USER app

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --version ^0.17 trunk

ENTRYPOINT ["./entrypoint"]
