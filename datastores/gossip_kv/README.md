# Gossip Key-Value Store

# Architecture
A gossip-based key-value store library.

```
┌─────────────────────────┐    ┌─────────────────────────┐    ┌─────────────────────────┐    ┌─────────────────────────┐
│        Process          │    │        Process          │    │        Process          │    │        Process          │
│ ┌─────────────────────┐ │    │ ┌─────────────────────┐ │    │ ┌─────────────────────┐ │    │ ┌─────────────────────┐ │
│ │  User Application   │ │    │ │  User Application   │ │    │ │   User Application  │ │    │ │   User Application  │ │
│ └────────▲──┬─────────┘ │    │ └─────────▲─┬─────────┘ │    │ └─────────▲─┬─────────┘ │    │ └─────────▲─┬─────────┘ │
│          │  │           │    │           │ │           │    │           │ │           │    │           │ │           │
│ ┌────────┴──▼─────────┐ │    │ ┌─────────┴─▼─────────┐ │    │ ┌─────────┴─▼─────────┐ │    │ ┌─────────┴─▼─────────┐ │
│ │   Gossip KV Store   │ │◄──►│ │   Gossip KV Store   │ │◄──►│ │   Gossip KV Store   │ │◄──►│ │   Gossip KV Store   │ │
│ └─────────────────────┘ │    │ └─────────────────────┘ │    │ └─────────────────────┘ │    │ └─────────────────────┘ │
└─────────────────────────┘    └─────────────────────────┘    └─────────────────────────┘    └─────────────────────────┘
```

## Data Model
TODO: Elaborate
* User Application manipulate data using client library
* Replicated to all members of the gossip cluster
* Eventually consistent

```json
{
  "sys": {
    "members": {
      "member_id": {
        "port": 1234,
        "protocol": "v1"
      }
    }
  },
  "usr": {
    "key 1": "value 1",
    "key 2": "value 2"
  }
}
```
Data in divided into two sections: A `sys` section that contains system data used by the key-value store itself. The
`usr` section contains user-defined data.

### `sys` Data
The `sys` data section contains system data / state that is required by the key-value store to do it's work.

#### Fields

### `usr` Data

## Protocol

## Checkpoints

# Running Locally Using Minikube
## Install Docker Desktop
```shell
brew install --cask docker
```
### Run docker (macOS)
```
open -a Docker
```

## Install Minikube
Read more [here](https://minikube.sigs.k8s.io/docs/start/)
```shell
brew install minikube
```

## Start Minikube
```shell
minikube start
```

## Install `kubectl`
```shell
brew install kubectl
```
## Configure Minikube to use your Docker Environment
```shell
eval $(minikube -p minikube docker-env)
```