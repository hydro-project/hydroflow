# Local (Minikube) Deployment

## Pre-requisites
- [Terraform](https://learn.hashicorp.com/tutorials/terraform/install-cli)
- [Minikube](https://minikube.sigs.k8s.io/docs/start/)
- [Docker](https://docs.docker.com/get-docker/)
- [Kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- [Make](https://www.gnu.org/software/make/)
- [k9s](https://k9scli.io/)

### Mac Setup Using Homebrew
```shell
brew install terraform
brew install minikube 
brew install docker
brew install kubernetes-cli
brew install k9s
```  

## Initialize Setup
Initializes terraform providers required for local deployment.
```shell
make init 
```

## Create Infrastructure
Creates a local kubernetes cluster using minikube.

```shell
make infra
```
### List Minikube Profiles
The `make infra` command creates minikube profile 'terraform-provider-minikube'. The details of the profile can be 
listed using the following command. 
```shell
minikube profile list
```
Other minikube commands can be specified using the profile name as shown below.
```shell
minikube --profile=terraform-provider-minikube <command>
```

### Kubectl
The kubectl context is set to the minikube profile 'terraform-provider-minikube' by default.

To switch back to the 'terraform-provider-minikube' context at anytime, use the following command.
```shell
kubectl config use-context terraform-provider-minikube
```

### K9s
To monitor the cluster using k9s, use the following command.

```shell
k9s
```

If the current context is not set to `terraform-provider-minikube`, use the following command.
```shell
k9s --context terraform-provider-minikube 
```

### Build Docker Image
Setup the docker environment to use the minikube docker registry.
```shell
eval $(minikube -p terraform-provider-minikube docker-env)
```

Build the docker image for the application, into the local minikube docker registry.
```shell
make docker_images
```
You can view the docker images in the minikube registry using the following command.
```shell
docker images
```

If docker isn't pointing to the minikube registry, use the following command.
```shell
minikube -p terraform-provider-minikube ssh docker images
```

## Create Application
```shell
make application
```

### Check if the application is running
```shell
kubectl get pods # All gossip-kv-* pods should show status as "Running"
````

### Update seed node configuration
```shell
make config
```

## Clean

Destroys the mini-kube cluster and removes the terraform state files.

```shell
make clean
```

If terraform is an inconsistent state, use the following command to blow away the minikube cluster and start fresh.
```shell
minikube delete -p terraform-provider-minikube
```