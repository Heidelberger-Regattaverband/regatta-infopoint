#!/bin/bash

docker build -t infopoint .

docker run -d --rm --name infopoint -p 80:8080 --env DB_PASSWORD=${DB_PASSWORD} infopoint