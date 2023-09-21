set dotenv-load := false
set export

NIGHTLY_VERSION := "2023-09-20"

push:
    docker buildx build --build-arg NIGHTLY_VERSION=$NIGHTLY_VERSION --build-arg PLATFORM=x86_64 --platform linux/amd64 -t "lyssieth/sauce-bot:latest" --push .
    docker buildx build --build-arg NIGHTLY_VERSION=$NIGHTLY_VERSION --build-arg PLATFORM=aarch64 --platform linux/arm64 -t "lyssieth/sauce-bot:latest" --push .

run:
    docker buildx build --build-arg NIGHTLY_VERSION=$NIGHTLY_VERSION --platform linux/amd64 -t lyssieth/sauce-bot .
    docker run -it -v /data/sauce-bot:/config lyssieth/sauce-bot
