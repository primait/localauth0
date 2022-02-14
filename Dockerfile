FROM public.ecr.aws/prima/rust:1.58.1

WORKDIR /code

COPY entrypoint /code/entrypoint

RUN chown -R app:app /code

# Needed to have the same file owner in the container and in Linux host
USER app

ENTRYPOINT ["/code/entrypoint"]
