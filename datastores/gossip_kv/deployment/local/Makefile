
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

.PHONY : init infra docker_images base_image server_image cli_image application config clean

init:
	terraform -chdir="$(INFRA_PATH)" init
	terraform -chdir="$(APPLICATION_PATH)" init

infra:
	terraform -chdir="$(INFRA_PATH)" apply -auto-approve

docker_images: base_image server_image cli_image

base_image:
	docker build -t "$(BASE_IMAGE_TAG)" -f ../../../../datastores/gossip_kv/server/baseimage.Dockerfile ../../../..

server_image:
	docker build -t "$(SERVER_IMAGE_TAG)" -f ../../../../datastores/gossip_kv/server/Dockerfile ../../../..

cli_image:
	docker build -t "$(CLI_IMAGE_TAG)" -f ../../../../datastores/gossip_kv/cli/Dockerfile ../../../..

application:
	terraform -chdir="$(APPLICATION_PATH)" apply -auto-approve

config:
	kubectl apply -f seed_node_config.yaml

clean:
	terraform -chdir="$(APPLICATION_PATH)" destroy -auto-approve
	terraform -chdir="$(INFRA_PATH)" destroy -auto-approve
	rm -rf $(INFRA_PATH)/.terraform $(INFRA_PATH)/terraform.tfstate $(INFRA_PATH)/terraform.tfstate.backup
	rm -rf $(APPLICATION_PATH)/.terraform $(APPLICATION_PATH)/terraform.tfstate $(APPLICATION_PATH)/terraform.tfstate.backup
