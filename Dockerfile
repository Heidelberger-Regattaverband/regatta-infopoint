FROM rust:1.63.0

WORKDIR /usr/src/infopoint
COPY . .

RUN cargo install --path .

CMD ["infopoint"]