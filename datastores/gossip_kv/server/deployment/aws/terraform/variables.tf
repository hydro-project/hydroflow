variable "region" {
  description = "AWS region where the resources will be created"
  type        = string
  default     = "us-east-2"
}

variable "instance_type" {
  description = "Instance type for the EC2 instances"
  type        = string
  default     = "t3.small"
}