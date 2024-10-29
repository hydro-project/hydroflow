output "cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value       = module.eks_cluster.cluster_endpoint
}

output "cluster_security_group_id" {
  description = "Security group ids attached to the cluster control plane"
  value       = module.eks_cluster.cluster_security_group_id
}

output "region" {
  description = "AWS region"
  value       = var.region
}

output "cluster_name" {
  description = "Kubernetes Cluster Name"
  value       = module.eks_cluster.cluster_name
}

output "repository_urls" {
  description = "URLs of all ECR repositories created"
  value       = { for repo, details in module.ecr : repo => details.repository_url }
}

output "grafana_port" {
  description = "Port for Grafana UI"
  value = var.grafana_port
}

output "prometheus_port" {
  description = "Port for Prometheus UI"
  value = var.prometheus_port
}