# Makefile

## Minikube Options.
MINIKUBE_DISK_SIZE:=100g
MINIKUBE_CPUS:=16
MINIKUBE_MEMORY:=32768

BASE_IMAGE_VERSION:=latest
SERVER_IMAGE_VERSION:=latest
CLI_IMAGE_VERSION:=latest

# Docker Image Tags
BASE_IMAGE_TAG:=hydroflow-gossip-kv-base-image:$(BASE_IMAGE_VERSION)
SERVER_IMAGE_TAG:=hydroflow-gossip-kv-server:$(SERVER_IMAGE_VERSION)
CLI_IMAGE_TAG:=hydroflow-gossip-kv-cli:$(CLI_IMAGE_VERSION)

AWS_TERRAFORM_PATH=../../datastores/gossip_kv/server/deployment/aws/terraform

# Define color variables
COLOR_RESET = \033[0m
COLOR_GREEN = \033[1;32m
COLOR_BLUE = \033[1;34m
COLOR_RED = \033[1;31m

# Define custom echo functions for different colors
greentext = @echo "$(COLOR_GREEN)$(1)$(COLOR_RESET)"
bluetext = @echo "$(COLOR_BLUE)$(1)$(COLOR_RESET)"
redtext = @echo "$(COLOR_RED)$(1)$(COLOR_RESET)"

aws_setup_cli:
	$(call cecho_green, Setting up kubectl to work with AWS EKS Cluster)
	aws eks update-kubeconfig --region $$(terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw region) --name $$(terraform -chdir=$(AWS_TERRAFORM​⬤

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

aws_terraform_init:
	terraform -chdir="$(AWS_TERRAFORM_PATH)" init

aws_terraform_apply:
	terraform -chdir="$(AWS_TERRAFORM_PATH)" apply

aws_setup_kubectl:
	@echo "Setting up kubectl to work with AWS EKS Cluster"
	aws eks update-kubeconfig --region $$(terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw region) --name $$(terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw cluster_name)

aws_upload_docker_images: build_docker_images
	$(eval SERVER_REPO_URL := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -json repository_urls | jq -r '.["gossip_kv_server"]'))
	$(eval CLI_REPO_URL := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -json repository_urls | jq -r '.["gossip_kv_cli"]'))
	$(eval REGION := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw region))
	docker tag $(SERVER_IMAGE_TAG) $(SERVER_REPO_URL):$(SERVER_IMAGE_VERSION)
	docker tag $(CLI_IMAGE_TAG) $(CLI_REPO_URL):$(CLI_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(SERVER_REPO_URL)
	docker push $(SERVER_REPO_URL):$(SERVER_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(SERVER_REPO_URL)
	docker push $(CLI_REPO_URL):$(CLI_IMAGE_VERSION)