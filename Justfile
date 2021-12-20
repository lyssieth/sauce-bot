set dotenv-load := false

push: build
    docker push lyssieth/sauce-bot:latest

build:
    docker buildx build -t "lyssieth/sauce-bot" .

run: build
    docker run -it -v /data/sauce-bot:/config lyssieth/sauce-bot
