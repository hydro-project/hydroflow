variable "grafana_port" {
  description = "Port for Grafana UI"
  type        = number
  default = 4001
}

variable "prometheus_port" {
  description = "Port for Prometheus UI"
  type = number
  default = 4002
}

variable "pod_monitoring_port" {
  description = "Port for monitoring pods using prometheus. Every pod runs a prometheus exporter on this port."
  type = number
  default = 4003
}