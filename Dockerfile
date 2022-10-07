# build image: docker build -t infopoint .
# run container: docker run -it --rm --name infopoint -p 80:8080 --env DB_PASSWORD= infopoint
FROM rust:1.64.0

LABEL maintainer="markus@ofterdinger.de"

RUN apt-get update && apt-get upgrade -y

WORKDIR /usr/src/infopoint
COPY . .

EXPOSE 8080

RUN cargo build --release
CMD ["./target/release/infopoint"]

#RUN cargo install --git https://github.com/Heidelberger-Regattaverband/regatta-infopoint.git --branch main
#CMD ["infopoint"]
