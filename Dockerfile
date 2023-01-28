# build image: docker build -t infoportal .
# run container: docker run -it --rm --name infoportal -p 80:8080 --env DB_PASSWORD= infoportal
FROM rust:1.67

LABEL maintainer="markus@ofterdinger.de"

# central configuration
WORKDIR /usr/src/infoportal
EXPOSE 8080

VOLUME [ "/data" ]

# copy required resources into image
COPY Cargo.toml .
COPY Cargo.lock .
COPY src/ ./src/
COPY static/ ./static/
COPY ssl/ ./ssl

# add node repository
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -

# install required software
RUN apt-get update && apt-get upgrade -y 
RUN rustup update stable
RUN apt-get install curl sudo nodejs -y

# build UI
RUN sudo npm install -g grunt-cli
RUN npm install --prefix ./static/
RUN grunt --gruntfile ./static/Gruntfile.js

# build rust application
RUN cargo build --release
CMD ["./target/release/infoportal"]

#RUN cargo install --git https://github.com/Heidelberger-Regattaverband/regatta-infopoint.git --branch main
#CMD ["infopoint"]
