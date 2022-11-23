# Hydroflow

Hydro's low-level dataflow runtime.

See the [Github Pages index](https://hydro-project.github.io/hydroflow/) for more documentation.

## Dev Setup

See the [setup section of the book](https://hydro-project.github.io/hydroflow/book/setup.html).

## The Examples Container

To make running Hydroflow's examples in the cloud easier, we've created a Docker image that contains compiled versions of those examples. The image is defined in the `Dockerfile` in the same directory as this README.

The `scripts/multiplatform-docker-build.sh <image name>` script will build both `arm64` and `amd64` versions of the image and push them to the image name specified. By default, this will push the image to DockerHub; if you want to push the image to another repository, you can pass an image URL as the argument to `multiplatform-docker-build.sh` instead.

Example binaries are located in `/usr/src/myapp`.
