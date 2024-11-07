variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-east-2"
}

variable "instance_type" {
  description = "Instance type for the EKS nodes"
  type        = string
  default     = "t3.small"
}

variable "ecr_repositories" {
  description = "List of ECR repository names"
  type        = list(string)
  default     = ["gossip_kv_server", "gossip_kv_cli", "gossip_kv_load_test"]
}