output "kubernetes_host" {
  value = minikube_cluster.gossip_kv.host
}

output "kubernetes_client_certificate" {
  value = minikube_cluster.gossip_kv.client_certificate
  sensitive = true
}

output "kubernetes_client_key" {
  value = minikube_cluster.gossip_kv.client_key
  sensitive = true
}

output "kubernetes_cluster_ca_certificate" {
  value = minikube_cluster.gossip_kv.cluster_ca_certificate
  sensitive = true
}