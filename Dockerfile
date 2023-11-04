# build image: docker build -t infoportal .
# run container: docker run -it --rm --name infoportal -p 8080:8080 -p 8443:8443 --env DB_HOST= --env DB_NAME= --env DB_USER= --env DB_PASSWORD= infoportal

ARG RUST_VERSION=1.73.0

#################
## build stage ##
#################
FROM rust:${RUST_VERSION} AS builder
LABEL maintainer="markus@ofterdinger.de"

# see https://github.com/hadolint/hadolint/wiki/DL4006
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN apt-get upgrade && apt-get update && apt-get install -y --no-install-recommends ca-certificates curl gnupg \
  && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
  && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list \
  && apt-get update && apt-get install -y --no-install-recommends nodejs \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && rustup update stable

WORKDIR /code

# copy required resources into builder image
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src/ src/
COPY static/ static/

# build rust application
RUN cargo fetch && cargo build --release

WORKDIR /code/static

# build UI5 application
RUN npm install && npx ui5 build --clean-dest

###############
## run stage ##
###############
FROM ubuntu:23.10
RUN apt-get update && apt-get upgrade -y \
  && apt-get install -y --no-install-recommends iputils-ping \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# copy server binary from build stage
COPY --from=builder /code/target/release/infoportal infoportal
COPY --from=builder /code/static/dist/ static/dist

# set user to non-root unless root is required for your app
USER 1001

EXPOSE 8080
EXPOSE 8443

CMD ["/app/infoportal"]
