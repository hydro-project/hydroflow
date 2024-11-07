INFRA_PATH=./terraform/infra
APPLICATION_PATH=./terraform/application

BASE_IMAGE_VERSION:=latest
SERVER_IMAGE_VERSION:=latest
CLI_IMAGE_VERSION:=latest
LOAD_TEST_IMAGE_VERSION:=latest

# Docker Image Tags
BASE_IMAGE_TAG:=hydroflow-gossip-kv-base-image:$(BASE_IMAGE_VERSION)
SERVER_IMAGE_TAG:=hydroflow-gossip-kv-server:$(SERVER_IMAGE_VERSION)
CLI_IMAGE_TAG:=hydroflow-gossip-kv-cli:$(CLI_IMAGE_VERSION)
LOAD_TEST_IMAGE_TAG:=hydroflow-gossip-kv-load-test:$(LOAD_TEST_IMAGE_VERSION)

.PHONY : init infra docker_images base_image server_image cli_image upload_docker_images application config clean

init:
	terraform -chdir="$(INFRA_PATH)" init
	terraform -chdir="$(APPLICATION_PATH)" init

infra:
	terraform -chdir="$(INFRA_PATH)" apply -auto-approve

kubectl_setup:
	@echo "Setting up kubectl to work with AWS EKS Cluster"
	aws eks update-kubeconfig --region $$(terraform -chdir=$(INFRA_PATH) output -raw region) --name $$(terraform -chdir=$(INFRA_PATH) output -raw cluster_name)

docker_images: base_image server_image cli_image

base_image:
	docker build -t "$(BASE_IMAGE_TAG)" -f ../../../../datastores/gossip_kv/server/baseimage.Dockerfile ../../../..

server_image:
	docker build -t "$(SERVER_IMAGE_TAG)" -f ../../../../datastores/gossip_kv/server/Dockerfile ../../../..

cli_image:
	docker build -t "$(CLI_IMAGE_TAG)" -f ../../../../datastores/gossip_kv/cli/Dockerfile ../../../..

upload_docker_images: docker_images
	$(eval SERVER_REPO_URL := $(shell terraform -chdir=$(INFRA_PATH) output -json repository_urls | jq -r '.["gossip_kv_server"]'))
	$(eval CLI_REPO_URL := $(shell terraform -chdir=$(INFRA_PATH) output -json repository_urls | jq -r '.["gossip_kv_cli"]'))
	$(eval LOAD_TEST_REPO_URL := $(shell terraform -chdir=$(INFRA_PATH) output -json repository_urls | jq -r '.["gossip_kv_load_test"]'))
	$(eval REGION := $(shell terraform -chdir=$(INFRA_PATH) output -raw region))
	echo $(SERVER_REPO_URL)
	docker tag $(SERVER_IMAGE_TAG) $(SERVER_REPO_URL):$(SERVER_IMAGE_VERSION)
	docker tag $(CLI_IMAGE_TAG) $(CLI_REPO_URL):$(CLI_IMAGE_VERSION)
	docker tag $(LOAD_TEST_IMAGE_TAG) $(LOAD_TEST_REPO_URL):$(LOAD_TEST_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(SERVER_REPO_URL)
	docker push $(SERVER_REPO_URL):$(SERVER_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(CLI_REPO_URL)
	docker push $(CLI_REPO_URL):$(CLI_IMAGE_VERSION)
	aws ecr get-login-password --region $(REGION) | docker login --username AWS --password-stdin $(LOAD_TEST_REPO_URL)
	docker push $(LOAD_TEST_REPO_URL):$(LOAD_TEST_IMAGE_VERSION)

application:
	terraform -chdir="$(APPLICATION_PATH)" apply -auto-approve

tunnel_grafana:
	$(eval GRAFANA_PORT := $(shell terraform -chdir=$(APPLICATION_PATH) output -raw grafana_port))
	@echo "Grafana will be accessible at http://localhost:$(GRAFANA_PORT)"
	kubectl port-forward svc/grafana $(GRAFANA_PORT):$(GRAFANA_PORT)

tunnel_prometheus:
	$(eval PROMETHEUS_PORT := $(shell terraform -chdir=$(APPLICATION_PATH) output -raw prometheus_port))
	@echo "Prometheus will be accessible at http://localhost:$(PROMETHEUS_PORT)"
	kubectl port-forward svc/prometheus $(PROMETHEUS_PORT):$(PROMETHEUS_PORT)


config:
	kubectl apply -f seed_node_config.yaml

clean:
	terraform -chdir="$(APPLICATION_PATH)" destroy -auto-approve
	terraform -chdir="$(INFRA_PATH)" destroy -auto-approve
	rm -rf $(INFRA_PATH)/.terraform $(INFRA_PATH)/terraform.tfstate $(INFRA_PATH)/terraform.tfstate.backup
	rm -rf $(APPLICATION_PATH)/.terraform $(APPLICATION_PATH)/terraform.tfstate $(APPLICATION_PATH)/terraform.tfstate.backup
