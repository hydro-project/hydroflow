terraform {
  backend "local" {
    path = "../state/application.tfstate"
  }
}

data "terraform_remote_state" "infra" {
  backend = "local"
  config = {
    path = "../state/infra.tfstate"
  }
}

provider "kubernetes" {
  host                   = data.terraform_remote_state.infra.outputs.kubernetes_host
  client_certificate     = data.terraform_remote_state.infra.outputs.kubernetes_client_certificate
  client_key             = data.terraform_remote_state.infra.outputs.kubernetes_client_key
  cluster_ca_certificate = data.terraform_remote_state.infra.outputs.kubernetes_cluster_ca_certificate
}

resource "kubernetes_stateful_set" "gossip_kv_seed_nodes" {
  metadata {
    name = "gossip-kv-seed-nodes"
    labels = {
      app = "gossip-kv-seed-nodes"
    }
  }

  spec {
    replicas    = 3
    service_name = "gossip-kv-seed-nodes"

    selector {
      match_labels = {
        app = "gossip-kv-seed-nodes"
      }
    }

    template {
      metadata {
        labels = {
          app = "gossip-kv-seed-nodes"
        }
      }
      spec {
        termination_grace_period_seconds = 5 # Allows for quick restarts. Not recommended for production.

        container {
          name  = "gossip-kv-server"
          image = "docker.io/hydroflow-gossip-kv-server:latest"
          image_pull_policy = "IfNotPresent"
          env {
            name  = "RUST_LOG"
            value = "trace"
          }
          env {
            name  = "RUST_BACKTRACE"
            value = "full"
          }
          port {
            container_port = 3001
            protocol       = "UDP"
          }
          volume_mount {
            name       = "gossip-kv-dynamic-config"
            mount_path = "/config/dynamic"
          }
        }

        volume {
          name = "gossip-kv-dynamic-config"
          config_map {
            name = "gossip-kv-dynamic-config"
          }
        }
      }
    }
  }
}

resource "kubernetes_deployment" "gossip_kv_cli" {
  metadata {
    name = "gossip-kv-cli"
    labels = {
      app = "gossip-kv-cli"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "gossip-kv-cli"
      }
    }

    template {
      metadata {
        labels = {
          app = "gossip-kv-cli"
        }
      }
      spec {
        termination_grace_period_seconds = 5
        container {
          name  = "gossip-kv-cli"
          image = "docker.io/hydroflow-gossip-kv-cli:latest"
          image_pull_policy = "IfNotPresent"
          command = ["/bin/sh"]
          args    = ["-c", "while true; do sleep 3600; done"]
          tty     = true
          env {
            name  = "RUST_LOG"
            value = "info"
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "gossip_kv_seed_nodes" {
  metadata {
    name = "gossip-kv-seed-nodes"
    labels = {
      app = "gossip-kv-seed-nodes"
    }
  }

  spec {
    port {
      port        = 3001
      target_port = 3001
      protocol    = "UDP"
    }
    cluster_ip = "None"
    selector = {
      app = "gossip-kv-seed-nodes"
    }
  }
}


resource "kubernetes_config_map" "gossip_kv_dynamic_config" {
  metadata {
    name = "gossip-kv-dynamic-config"
  }

  data = {
    "dynamic.toml" = ""
  }
}