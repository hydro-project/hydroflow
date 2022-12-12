# Hydroflow

Hydro's low-level dataflow runtime.

See the [Github Pages index](https://hydro-project.github.io/hydroflow/) for more documentation.

Read the [Hydroflow Book](https://hydro-project.github.io/hydroflow/book/).

## Dev Setup

See the [setup section of the book](https://hydro-project.github.io/hydroflow/book/setup.html).

## The Examples Container

To make running Hydroflow's examples in the cloud easier, we've created a Docker image that contains compiled versions of those examples. The image is defined in the `Dockerfile` in the same directory as this README.

If you want to build the examples container locally, you can run

```
docker build -t hydroflow-examples .
```

This will build an image suitable for your architecture.

The `scripts/multiplatform-docker-build.sh <image name>` script will build both `arm64` and `amd64` versions of the image and push them to the image name specified. By default, this will push the image to DockerHub; if you want to push the image to another repository, you can pass an image URL as the argument to `multiplatform-docker-build.sh` instead.

Example binaries are located in `/usr/src/myapp`.

## mdBook Setup

[The Hydroflow Book](https://hydro-project.github.io/hydroflow/book/) is generated using [mdBook](https://rust-lang.github.io/mdBook/). To install `mdbook` and dependencies:
```bash
cargo install mdbook mdbook-mermaid mdbook-linkcheck
```
The book can then be viewed locally with a web browser by running the following from the project root.
```bash
mdbook serve --open
```
