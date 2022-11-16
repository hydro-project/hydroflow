#!/bin/bash

docker buildx create --use --name hydroflow-multiplatform --node hydroflow-multiplatform

docker buildx build --builder hydroflow-multiplatform --cache-to type=inline --platform linux/arm64,linux/amd64 -t hydro-project/hydroflow .
