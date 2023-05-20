#!/bin/bash

git pull -p

if [[ -z ${DB_PASSWORD} ]]; then
  echo "Environment variable DB_PASSWORD not set."
  exit 1
fi
docker-compose build

docker-compose up -d

docker logs infoportal -f

#docker system prune
