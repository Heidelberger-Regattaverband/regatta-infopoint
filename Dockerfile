# build image: docker build -t infoportal .
# run container: docker run -it --rm --name infoportal -p 8080:8080 -p 8443:8443 --env DB_PASSWORD= infoportal

#################
## build stage ##
#################
FROM rust:1.67.1 AS builder
LABEL maintainer="markus@ofterdinger.de"

# add node repository
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -

# install required software
RUN apt-get update && apt-get upgrade -y
RUN rustup update stable
RUN apt-get install curl sudo nodejs -y
RUN sudo npm install -g grunt-cli

WORKDIR /code

# copy required resources into builder image
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src/ src/
COPY ssl/ ssl
COPY .env .env

RUN cargo fetch

COPY static/ static/

# build UI
RUN npm install --prefix ./static/
RUN grunt --gruntfile ./static/Gruntfile.js

# build rust application
RUN cargo build --release

###############
## run stage ##
###############
FROM debian:bullseye-slim
WORKDIR /app

# copy server binary from build stage
COPY --from=builder /code/target/release/infoportal ./infoportal
COPY --from=builder /code/.env ./.env
COPY --from=builder /code/static/infoportal/ ./static/infoportal/
COPY --from=builder /code/ssl/ ./ssl/

# set user to non-root unless root is required for your app
USER 1001

EXPOSE 8080
EXPOSE 8443
VOLUME [ "/data" ]

CMD ["/app/infoportal"]

#RUN cargo install --git https://github.com/Heidelberger-Regattaverband/regatta-infopoint.git --branch main
#CMD ["infopoint"]
