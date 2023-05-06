#!/bin/bash

git pull -p

docker-compose build

docker-compose up -d

docker logs infoportal -f

docker system prune
