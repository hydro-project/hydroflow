output "cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value = module.eks_cluster.cluster_endpoint
}

output "cluster_name" {
  description = "Kubernetes Cluster Name"
  value = module.eks_cluster.cluster_name
}

output "vpc_id" {
  value = module.vpc.vpc_id
}

output "region" {
  description = "AWS region"
  value       = var.region
}

output "repository_urls" {
  description = "URLs of all ECR repositories created"
  value       = { for repo, details in module.ecr : repo => details.repository_url }
}

output "cluster_certificate_authority_data" {
  description = "Base64 encoded PEM certificate data for the cluster"
  value       = module.eks_cluster.cluster_certificate_authority_data
}