FROM public.ecr.aws/prima/rust:1.60.0

WORKDIR /code

COPY entrypoint /code/entrypoint

RUN chown -R app:app /code && \
    rustup target add wasm32-unknown-unknown && \
    cargo install --locked trunk

# Needed to have the same file owner in the container and in Linux host
USER app

ENTRYPOINT ["./entrypoint"]
