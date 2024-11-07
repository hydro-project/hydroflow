terraform {
  backend "local" {
    path = "../state/infra.tfstate"
  }

  required_providers {
    minikube = {
      source  = "scott-the-programmer/minikube"
      version = "0.4.2"
    }
  }
}

resource "minikube_cluster" "gossip_kv" {
  driver    = "docker"
  cpus      = 16
  memory    = 32768
  disk_size = "100g"
}

provider "kubernetes" {
  host                   = minikube_cluster.gossip_kv.host
  client_certificate     = minikube_cluster.gossip_kv.client_certificate
  client_key             = minikube_cluster.gossip_kv.client_key
  cluster_ca_certificate = minikube_cluster.gossip_kv.cluster_ca_certificate
}
