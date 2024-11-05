# AWS Deployment 

## Pre-requisites
- [Terraform](https://learn.hashicorp.com/tutorials/terraform/install-cli)
- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-install.html)
- [Docker](https://docs.docker.com/get-docker/)
- [Kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- [Make](https://www.gnu.org/software/make/)
- [k9s](https://k9scli.io/)

### Mac Setup Using Homebrew
```shell
brew install terraform
brew install awscli
brew install docker
brew install kubernetes-cli
brew install k9s
```  

## Initialize Setup
Initializes terraform providers required for local deployment.
```shell
make init 
```

## Configure AWS Credentials
Make sure you have the AWS credentials configured. Subsequent terraform commands will use these credentials to create 
the infrastructure.
```shell
aws configure
```

## Create Infrastructure
Creates a local kubernetes cluster using minikube.

```shell
make infra
```

## Kubectl
To enable kubectl usage with the newly created AWS EKS cluster, use the following command.

```shell
make kubectl_setup
```

## K9s
To monitor the cluster using k9s, use the following command.

```shell
k9s
```

## Build & Upload Docker Image
Build and upload the docker image for the application, into ECR.
```shell
make upload_docker_images
```

## Create Application
```shell
make application
```

## Check if the application is running
```shell
kubectl get pods # All gossip-kv-* pods should show status as "Running"
````

## Access Grafana Dashboards
```shell
make tunnel_grafana
```

## Access Prometheus
```shell
make tunnel_prometheus
```

## Update seed node configuration
```shell
make config
```

## Clean
Destroys the AWS resources and cleans up terraform state.
```shell
make clean
```