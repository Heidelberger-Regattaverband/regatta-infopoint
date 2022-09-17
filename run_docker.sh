docker build -t infopoint .

docker run -it --rm --name infopoint -p 80:8080 --env DB_PASSWORD= infopoint