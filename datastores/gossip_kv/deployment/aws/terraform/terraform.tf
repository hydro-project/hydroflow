terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.70.0"
    }

    random = {
      source  = "hashicorp/random"
      version = "~> 3.6.1"
    }

    tls = {
      source  = "hashicorp/tls"
      version = "~> 4.0.5"
    }

    cloudinit = {
      source  = "hashicorp/cloudinit"
      version = "~> 2.3.4"
    }

    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = ">= 2.32.0"
    }
  }

  required_version = "~> 1.3"
}