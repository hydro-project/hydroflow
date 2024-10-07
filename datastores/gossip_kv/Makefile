# Makefile

## Minikube Options.
MINIKUBE_DISK_SIZE=100g
MINIKUBE_CPUS=16
MINIKUBE_MEMORY=32768

# Docker Image Tags
BASE_IMAGE_TAG=hydroflow/gossip-kv-base-image:latest
SERVER_IMAGE_TAG=hydroflow/gossip-kv-server:latest
CLI_IMAGE_TAG=hydroflow/gossip-kv-cli:latest
DUMMY_IMAGE_TAG=hydroflow/gossip-kv-dummy:latest

# Default target when you run 'make'

# Target to start Minikube with specific options
start_minikube:
	minikube start --disk-size=$(MINIKUBE_DISK_SIZE) --cpus=$(MINIKUBE_CPUS) --memory=$(MINIKUBE_MEMORY)
	@echo "Please run 'eval \$$(minikube docker-env)' to use the Minikube Docker daemon"

# Target to build the Docker images
build_docker_images: build_base_image build_server_image build_cli_image

build_base_image:
	docker build -t "$(BASE_IMAGE_TAG)" -f ../../datastores/gossip_kv/server/baseimage.Dockerfile ../..

build_server_image:
	docker build -t "$(SERVER_IMAGE_TAG)" -f ../../datastores/gossip_kv/server/Dockerfile ../..

build_cli_image:
	docker build -t "$(CLI_IMAGE_TAG)" -f ../../datastores/gossip_kv/cli/Dockerfile ../..

# Target to clean up the Minikube cluster
clean_local:
	minikube delete

# Target to deploy the Gossip KV Server to the Minikube cluster
deploy_local:
	kubectl apply -f ../../datastores/gossip_kv/server/local

# Target to delete the Minikube cluster and build again
rebuild_local: clean_local start_minikube build_docker_images