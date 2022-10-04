#!/bin/bash

if [ -z ${DB_PASSWORD} ]
then
  echo "\$DB_PASSWORD is empty, exiting image creation."
  exit 1
else
  echo "Creating docker image with given variables."
fi

if [ -z ${RUST_LOG} ]
then
  echo "\$RUST_LOG is empty, setting it to INFO."
  RUST_LOG=INFO
else
  echo "Using RUST_LOG=${RUST_LOG}"
fi

git pull -p

docker build -t infopoint .

docker stop infopoint

docker run -d --rm --name infopoint -p 80:8080 --env DB_PASSWORD=${DB_PASSWORD} --env RUST_LOG=${RUST_LOG} infopoint

docker logs infopoint --follow
