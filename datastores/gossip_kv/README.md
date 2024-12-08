# Gossip Key-Value Store

# Architecture
The Gossip Key-Value Store is a distributed key-value store that uses a gossip protocol to replicate data across all
members of the cluster. The key-value store is eventually consistent and is designed to be used in scenarios where 
strict consistency is not a hard requirement.

## Data Model
Data stored in the key-value store is modelled as three level hierarchy `Namespace > Table > Rows`. 

```
┌──────────────┐              
│  Namespace   │             
└─┬────────────┘              
  │    ┌──────────────┐       
  └────┤    Table     │       
       └─┬────────────┘       
         │    ┌──────────────┐
         └────┤     Row      │
              └──────────────┘
```

Represented as JSON, the data model looks like this:

```json
{
  "sys": {
    "members": {
      "member_id": {
        "protocols": [
          {
            "name": "gossip",
            "endpoint": "..."
          },
          {
            "name": "...",
            "endpoint": "..."
          }
        ]
      }
    }
  },
  "usr": {
    "table_A": {
      "key_1": "value_1",
      "key_2": "value_2"
    },
    "table_B": {
      "key_1": "value_1",
      "key_2": "value_2"
    }
  }
}
```
Note that JSON is used here for illustrative purposes only - it isn't the actual data format used by the key-value 
store.

### Namespace
A namespace is a group of tables. There are only two namespaces in the key-value store: `sys` and `usr`. The `sys` 
namespace is reserved for system data and is used by the key-value store itself. The `usr` namespace is used for user
data.

### `sys` Data

### Members
The members table contains information about all the members in the cluster. Each member is identified by a unique
`member_id`.

#### Protocols
Each member exposes a set of protocols and endpoint that can be used to communicate with it. For example, every member
exposes the `gossip` protocol which is used by the key-value store to replicate data across the cluster.

### `usr` Data
The `usr` namespace is used to store user data. The data is stored in tables and rows. Each table is a collection of rows
where each row is a key-value pair. Values are stored as strings and are not interpreted by the key-value store.

# Unit Testing
A deterministic unit test suite is provided for the key-value store and the replication protocols. See the unit tests 
on [server.rs](./kv/server.rs) for more.

# Deployment
## Local (Minikube) Deployment
For local development and testing, the key-value store can be deployed on Minikube. See [Minikube Deployment](./deployment/local/README.md) 
for more information.

## AWS Deployment
See [AWS Deployment](./deployment/aws/README.md) for more information. 