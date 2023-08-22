FROM public.ecr.aws/prima/rust:1.71.0

WORKDIR /code

COPY entrypoint /code/entrypoint

RUN chown -R app:app /code
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

# Needed to have the same file owner in the container and in Linux host
USER app

ENTRYPOINT ["./entrypoint"]
