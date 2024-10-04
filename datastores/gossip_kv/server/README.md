From the `hydroflow` directory, run 

## Minikube

### Start Minikube
Disk allocation is done by the driver used to create the VM. Setting this to a high value will do nothing if the
driver isn't correctly configured. You'll only notice that hydroflow runs out of disk space while compiling.
For Docker, the disk size is set in the Docker Desktop settings. Also, provide as many CPUs here as possible, since 
building the code is CPU-intensive.
```shell
minikube start --disk-size=100g --cpus=16 --memory=32768
```

### Use the Docker daemon from minikube
```shell
eval $(minikube docker-env)
``` 

## Build Docker Base Image
Speeds up code changes by caching build dependencies.
```shell
docker build -t "hydroflow/gossip-kv-server-base-image:latest" -f datastores/gossip_kv/server/baseimage.Dockerfile . 
```

## Build Docker Image for Gossip Server
```shell
docker build -t "hydroflow/gossip-kv-server:latest" -f datastores/gossip_kv/server/Dockerfile .
```

## Build Docker Image for Gossip CLI
```shell
docker build -t "hydroflow/gossip-kv-cli:latest" -f datastores/gossip_kv/cli/Dockerfile .
```

## Check if minikube has the image
You should see "hydroflow/gossip-kv"
```shell
minikube image ls --format tablemin
```

## Deploy to Minikube
```shell
kubectl apply -f datastores/gossip_kv/server/deployment/local
```