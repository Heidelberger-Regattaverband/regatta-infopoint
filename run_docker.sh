#!/bin/bash

if [ -z ${DB_PASSWORD} ]
then
  echo "\$DB_PASSWORD is empty, exiting image creation."
  exit 1
else
  echo "Creating docker image with given variables."
fi

git pull -p

docker build -t infopoint .

docker stop infopoint

docker run -d --rm --name infopoint -p 80:8080 --env DB_PASSWORD=${DB_PASSWORD} infopoint