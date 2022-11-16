#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: multiplatform-docker-build.sh <image name>"
    exit 1
fi

IMAGE_NAME=$1

docker buildx create --use --name hydroflow-multiplatform --node hydroflow-multiplatform

docker buildx build --builder hydroflow-multiplatform --cache-to type=inline --platform linux/arm64,linux/amd64 -t ${IMAGE_NAME} --push .
