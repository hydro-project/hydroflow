From the `datastores/gossip_kv` directory, run 

## Build Docker Base Image
Speeds up code changes by caching build dependencies.
```shell
docker build -t "hydroflow/gossip-kv-server-base-image:latest" -f examples/server/baseimage.Dockerfile . 
```

## Build Docker Server Image
```shell
docker build -t "hydroflow/gossip-kv-server:latest" -f examples/server/Dockerfile . 
```

## Check if minikube has the image
You should see "hydroflow/gossip-kv"
```shell
minikube image ls --format table
``

## Deploy to Minikube
```shell
kubectl apply -f examples/server/deployment/local
```