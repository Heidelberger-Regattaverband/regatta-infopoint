# build image: docker build -t infopoint .
# run container: docker run -it --rm --name infopoint -p 8080:8080 --env DB_PASSWORD= infopoint
FROM rust:1.63.0

WORKDIR /usr/src/infopoint
COPY . .

EXPOSE 8080
ENV RUST_LOG=INFO

#RUN cargo build --release
#CMD ["./target/release/infopoint"]

RUN cargo install --git https://github.com/Heidelberger-Regattaverband/regatta-infopoint.git --branch main
CMD ["infopoint"]
