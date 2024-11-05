provider "aws" {
  region = var.region
}

# Filter out local zones, which are not currently supported
# with managed node groups
data "aws_availability_zones" "available" {
  filter {
    name = "opt-in-status"
    values = ["opt-in-not-required"]
  }
}

data "aws_caller_identity" "current" {}

locals {
  cluster_name = "anna-load-test-${random_string.suffix.result}"
  account_id   = data.aws_caller_identity.current.account_id
}

resource "random_string" "suffix" {
  length  = 8
  special = false
}

module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.8.1"

  name = "anna-load-test-vpc"

  cidr = "10.0.0.0/16"
  azs = slice(data.aws_availability_zones.available.names, 0, 3)

  map_public_ip_on_launch = true
  public_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]

  enable_dns_hostnames = true

  public_subnet_tags = {
    "kubernetes.io/role/elb" = 1
  }
}

module "eks_cluster" {
  source  = "terraform-aws-modules/eks/aws"
  version = "20.24.3"

  cluster_name    = local.cluster_name
  cluster_version = "1.31"

  cluster_endpoint_public_access           = true
  enable_cluster_creator_admin_permissions = true

  cluster_addons = {
    aws-ebs-csi-driver = {
      service_account_role_arn = module.irsa-ebs-csi.iam_role_arn
    }
  }

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.public_subnets

  eks_managed_node_group_defaults = {
    ami_type = "AL2_x86_64"
  }

  eks_managed_node_groups = {
    one = {
      name = "servers"

      instance_types = [var.instance_type]

      min_size     = 1
      max_size     = 3
      desired_size = 2
    }
  }
}

# https://aws.amazon.com/blogs/containers/amazon-ebs-csi-driver-is-now-generally-available-in-amazon-eks-add-ons/
data "aws_iam_policy" "ebs_csi_policy" {
  arn = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
}

module "irsa-ebs-csi" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-assumable-role-with-oidc"
  version = "5.39.0"

  create_role  = true
  role_name    = "AmazonEKSTFEBSCSIRole-${module.eks_cluster.cluster_name}"
  provider_url = module.eks_cluster.oidc_provider
  role_policy_arns = [data.aws_iam_policy.ebs_csi_policy.arn]
  oidc_fully_qualified_subjects = ["system:serviceaccount:kube-system:ebs-csi-controller-sa"]
}

variable "ecr_repositories" {
  description = "List of ECR repository names"
  type = list(string)
  default = ["gossip_kv_server", "gossip_kv_cli", "gossip_kv_load_test"]
}

module "ecr" {
  source  = "terraform-aws-modules/ecr/aws"
  version = "2.3.0"

  for_each        = {for repo in var.ecr_repositories : repo => repo}
  repository_name = each.value

  repository_read_write_access_arns = [data.aws_caller_identity.current.arn]
  repository_lifecycle_policy = jsonencode({
    rules = [
      {
        rulePriority = 1,
        description  = "Keep last 30 images",
        selection = {
          tagStatus   = "tagged",
          tagPrefixList = ["v"],
          countType   = "imageCountMoreThan",
          countNumber = 30
        },
        action = {
          type = "expire"
        }
      }
    ]
  })

  repository_image_tag_mutability = "MUTABLE"
  tags = {
    Terraform   = "true"
    Environment = "dev"
  }
}

provider "kubernetes" {
  host = module.eks_cluster.cluster_endpoint
  cluster_ca_certificate = base64decode(module.eks_cluster.cluster_certificate_authority_data)
  exec {
    api_version = "client.authentication.k8s.io/v1beta1"
    command     = "aws"
    args = [
      "eks",
      "get-token",
      "--cluster-name",
      module.eks_cluster.cluster_name,
    ]
  }
}

resource "kubernetes_stateful_set" "gossip_kv_seed_nodes" {
  metadata {
    name = "gossip-kv-seed-nodes"
    labels = {
      app = "gossip-kv-seed-nodes"
    }
  }

  spec {
    service_name = "gossip-kv-seed-nodes"
    replicas     = 1

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
        annotations = {
          "prometheus.io/scrape" : "true"
          "prometheus.io/port" : var.pod_monitoring_port
        }
      }

      spec {
        termination_grace_period_seconds = 5

        container {
          name              = "gossip-kv-server"
          image             = "${module.ecr.gossip_kv_load_test.repository_url}:latest"
          image_pull_policy = "Always"

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

          port {
            container_port = var.pod_monitoring_port
            protocol       = "TCP"
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
          name              = "gossip-kv-cli"
          image             = "${module.ecr.gossip_kv_cli.repository_url}:latest"
          image_pull_policy = "Always"
          command = ["/bin/sh"]
          args = ["-c", "while true; do sleep 3600; done"]
          tty               = true

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
    cluster_ip = "None"
    selector = {
      app = "gossip-kv-seed-nodes"
    }

    port {
      port        = 3001
      target_port = 3001
      protocol    = "UDP"
    }
  }
}

resource "kubernetes_config_map" "gossip_kv_dynamic_config" {
  metadata {
    name = "gossip-kv-dynamic-config"
  }

  data = {
    "dynamic.toml" = <<EOF
    # Your dynamic TOML configuration here
    EOF
  }
}

resource "kubernetes_config_map" "grafana_datasource" {
  metadata {
    name = "grafana-datasource"
    namespace = "default"
  }

  data = {
    "prometheus-datasource.yaml" = <<EOF
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus.default.svc:4002
    isDefault: true
    version: 1
    editable: false
EOF
  }
}

resource "kubernetes_deployment" "grafana" {
  metadata {
    name = "grafana"
    labels = {
      app = "grafana"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "grafana"
      }
    }

    template {
      metadata {
        labels = {
          app = "grafana"
        }
      }

      spec {
        container {
          name  = "grafana"
          image = "grafana/grafana:latest"

          port {
            container_port = var.grafana_port
          }

          env {
            name  = "GF_SECURITY_ADMIN_PASSWORD"
            value = "admin" # Change this password
          }

          env {
            name = "GF_SERVER_HTTP_PORT"
            value = var.grafana_port
          }

          env {
            name = "GF_AUTH_ANONYMOUS_ENABLED"
            value = "true"
          }

          env {
            name = "GF_AUTH_ANONYMOUS_ORG_ROLE"
            value = "Admin"
          }

          env {
            name = "GF_SECURITY_DISABLE_INITIAL_ADMIN_PASSWORD_CHANGE"
            value = "true"
          }

          volume_mount {
            name       = "grafana-datasource"
            mount_path = "/etc/grafana/provisioning/datasources"
          }
        }

        volume {
          name = "grafana-datasource"

          config_map {
            name = kubernetes_config_map.grafana_datasource.metadata[0].name
          }
        }
      }
    }
  }
}
resource "kubernetes_service" "grafana" {
  metadata {
    name = "grafana"
  }

  spec {
    selector = {
      app = "grafana"
    }

    port {
      port        = var.grafana_port
      target_port = var.grafana_port
    }

    type = "ClusterIP" # Use "LoadBalancer" if external access is needed
  }
}

resource "kubernetes_service_account" "prometheus_sa" {
  metadata {
    name      = "prometheus-sa"
    namespace = "default"
  }
}

resource "kubernetes_cluster_role" "prometheus_cluster_role" {
  metadata {
    name = "prometheus-cluster-role"
  }

  rule {
    api_groups = [""]
    resources = ["pods", "services", "endpoints", "nodes"]
    verbs = ["get", "list", "watch"]
  }

  rule {
    api_groups = ["extensions", "networking.k8s.io"]
    resources = ["ingresses", "networkpolicies"]
    verbs = ["get", "list", "watch"]
  }
}

resource "kubernetes_cluster_role_binding" "prometheus_cluster_role_binding" {
  metadata {
    name = "prometheus-cluster-role-binding"
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.prometheus_cluster_role.metadata[0].name
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.prometheus_sa.metadata[0].name
    namespace = kubernetes_service_account.prometheus_sa.metadata[0].namespace
  }
}


resource "kubernetes_config_map" "prometheus_config" {
  metadata {
    name      = "prometheus-config"
    namespace = "default"
  }

  data = {
    "prometheus.yml" = <<EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        target_label: __meta_kubernetes_pod_container_port_number
        replacement: "$1"
EOF
  }
}

resource "kubernetes_deployment" "prometheus" {
  metadata {
    name      = "prometheus"
    namespace = "default"
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "prometheus"
      }
    }

    template {
      metadata {
        labels = {
          app = "prometheus"
        }
      }

      spec {
        service_account_name = kubernetes_service_account.prometheus_sa.metadata[0].name

        container {
          name  = "prometheus"
          image = "prom/prometheus:latest"
          args = ["--config.file=/etc/prometheus/prometheus.yml", "--web.listen-address=:${var.prometheus_port}"]

          volume_mount {
            name       = "config-volume"
            mount_path = "/etc/prometheus"
          }

          port {
            container_port = var.prometheus_port  # Set to your desired port
          }
        }

        volume {
          name = "config-volume"
          config_map {
            name = kubernetes_config_map.prometheus_config.metadata[0].name
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "prometheus" {
  metadata {
    name      = "prometheus"
    namespace = "default"
  }

  spec {
    selector = {
      app = "prometheus"
    }

    port {
      port        = var.prometheus_port
      target_port = var.prometheus_port
    }

    type = "ClusterIP"  # Use "LoadBalancer" if external access is needed
  }
}