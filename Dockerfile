# build image: docker build -t infopoint .
# run container: docker run -it --rm --name infopoint -p 8080:8080 --env DB_PASSWORD= infopoint

FROM rust:1.63.0

WORKDIR /usr/src/infopoint
COPY . .

#RUN cargo install --path .
#CMD ["infopoint"]

RUN cargo build --release

ENV RUST_LOG=INFO
#ENV HTTP_BIND=0.0.0.0

EXPOSE 8080

CMD ["./target/release/infopoint"]