# Makefile

## Minikube Options.
MINIKUBE_DISK_SIZE:=100g
MINIKUBE_CPUS:=16
MINIKUBE_MEMORY:=32768

BASE_IMAGE_VERSION:=latest
SERVER_IMAGE_VERSION:=latest
CLI_IMAGE_VERSION:=latest
LOAD_TEST_IMAGE_VERSION:=latest

# Docker Image Tags
BASE_IMAGE_TAG:=hydroflow-gossip-kv-base-image:$(BASE_IMAGE_VERSION)
SERVER_IMAGE_TAG:=hydroflow-gossip-kv-server:$(SERVER_IMAGE_VERSION)
CLI_IMAGE_TAG:=hydroflow-gossip-kv-cli:$(CLI_IMAGE_VERSION)
LOAD_TEST_IMAGE_TAG:=hydroflow-gossip-kv-load-test:$(LOAD_TEST_IMAGE_VERSION)

AWS_TERRAFORM_PATH=../../datastores/gossip_kv/deployment/aws/terraform

# Target to start Minikube with specific options
start_minikube:
	minikube start --disk-size=$(MINIKUBE_DISK_SIZE) --cpus=$(MINIKUBE_CPUS) --memory=$(MINIKUBE_MEMORY)
	@echo "Please run 'eval \$$(minikube docker-env)' to use the Minikube Docker daemon"

# Target to build the Docker images
build_docker_images: build_base_image build_server_image build_cli_image build_load_test_image

build_base_image:
	docker build -t "$(BASE_IMAGE_TAG)" -f ../../datastores/gossip_kv/server/baseimage.Dockerfile ../..

build_server_image:
	docker build -t "$(SERVER_IMAGE_TAG)" -f ../../datastores/gossip_kv/server/Dockerfile ../..

build_cli_image:
	docker build -t "$(CLI_IMAGE_TAG)" -f ../../datastores/gossip_kv/cli/Dockerfile ../..

build_load_test_image:
	docker build -t "$(LOAD_TEST_IMAGE_TAG)" -f ../../datastores/gossip_kv/load_test_server/Dockerfile ../..

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
	$(eval LOAD_TEST_REPO_URL := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -json repository_urls | jq -r '.["gossip_kv_load_test"]'))
	$(eval REGION := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw region))
	docker tag $(SERVER_IMAGE_TAG) $(SERVER_REPO_URL):$(SERVER_IMAGE_VERSION)
	docker tag $(CLI_IMAGE_TAG) $(CLI_REPO_URL):$(CLI_IMAGE_VERSION)
	docker tag $(LOAD_TEST_IMAGE_TAG) $(LOAD_TEST_REPO_URL):$(LOAD_TEST_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(SERVER_REPO_URL)
	docker push $(SERVER_REPO_URL):$(SERVER_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(CLI_REPO_URL)
	docker push $(CLI_REPO_URL):$(CLI_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(LOAD_TEST_REPO_URL)
	docker push $(LOAD_TEST_REPO_URL):$(LOAD_TEST_IMAGE_VERSION)

aws_tunnel_grafana:
	$(eval GRAFANA_PORT := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw grafana_port))
	kubectl port-forward svc/grafana $(GRAFANA_PORT):$(GRAFANA_PORT)

aws_tunnel_prometheus:
	$(eval PROMETHEUS_PORT := $(shell terraform -chdir=$(AWS_TERRAFORM_PATH) output -raw prometheus_port))
	kubectl port-forward svc/prometheus $(PROMETHEUS_PORT):$(PROMETHEUS_PORT)