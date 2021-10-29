FROM rust

WORKDIR /app

RUN cargo install cargo-watch

ENV PORT=8080
EXPOSE 8080

CMD cargo watch -x run
