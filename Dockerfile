# build image: docker build -t infoportal .
# run container: docker run -it --rm --name infoportal -p 8080:8080 -p 8443:8443 --env DB_HOST= --env DB_NAME= --env DB_USER= --env DB_PASSWORD= infoportal

ARG RUST_VERSION=1.73.0

#################
## build stage ##
#################
FROM rust:${RUST_VERSION} AS builder
LABEL maintainer="markus@ofterdinger.de"

ARG NODE_VERSION=18

# see https://github.com/hadolint/hadolint/wiki/DL4006
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# add node repository
RUN curl -fsSL "https://deb.nodesource.com/setup_${NODE_VERSION}.x" | bash -

RUN rustup update stable \
 && apt-get update && apt-get upgrade -y \
 && apt-get install -y --no-install-recommends curl sudo nodejs \
 && sudo npm install -g grunt-cli

WORKDIR /code

# copy required resources into builder image
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src/ src/
COPY ssl/ ssl
COPY .env .env

RUN cargo fetch

COPY static/ static/

# build rust application
RUN cargo build --release \
 && npm install --prefix ./static/ \
 && grunt --gruntfile ./static/Gruntfile.js

###############
## run stage ##
###############
FROM ubuntu:23.04
RUN apt-get update && apt-get upgrade -y \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# copy server binary from build stage
COPY --from=builder /code/target/release/infoportal infoportal
COPY --from=builder /code/.env .env
COPY --from=builder /code/static/webapp/ static/webapp
COPY --from=builder /code/ssl/ ssl

# set user to non-root unless root is required for your app
USER 1001

EXPOSE 8080
EXPOSE 8443
VOLUME [ "/data" ]

CMD ["/app/infoportal"]
