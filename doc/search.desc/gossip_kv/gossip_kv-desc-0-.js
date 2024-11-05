searchState.loadedDescShard("gossip_kv", 0, "An acknowledgement message sent by a peer in response to a …\nA request from a client to the key-value store.\nA request to delete the value of a key.\nA response for a delete request. The success field is true …\nA request to get the value of a key.\nA response for a get request. The key is echoed back along …\nAn “infecting message” to share updates with a peer.\nThe key is in an invalid format. Keys must be of the form …\nThe namespace in the key is invalid. Namespaces must be …\nA key of an entry in the key-value store.\nError that can occur when parsing a key from a string.\nA negative acknowledgement sent by a peer in response to a …\nThe namespace of the key of an entry in the key-value …\nThe key of a row in a table in the key-value store.\nA request to set the value of a key.\nA response for a set request. The success field is true if …\nSystem namespace is reserved for use by the key-value …\nThe name of a table in the key-value store.\nUser namespace is for use by the user of the key-value …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe namespace of the key.\nThe key of the row in the table.\nThe name of the table in the key.\nA bounded set union lattice with a fixed size N.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nInformation about a member in the cluster.\nA builder for <code>MemberData</code>.\nA protocol supported by a member.\nAdds a protocol to the member.\nBuilds the <code>MemberData</code>.\nThe endpoint on which the protocol is running.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nThe name of the member. Usually, this is a randomly …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe name of the protocol.\nCreates a new <code>MemberDataBuilder</code>.\nCreates a new <code>Protocol</code>.\nThe protocols that the member supports.\nTimestamps used in the model.\nPrimary key for entries in a table.\nValue stored in a table. Modelled as a timestamped set of …\nA map from row keys to values in a table.\nA map from table names to tables.\nName of a table in the data store.\nTableMap element to delete a row from an existing TableMap.\nThe <code>Key</code> of the  dominating pair lattice, usually a …\nTableMap element to upsert a row in an existing TableMap.\nA trait that represents an abstract network address. In …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates a L0 key-value store server using Hydroflow.\nAn ack request with the message id and the address of the …\nConvenience enum to represent a client request with the …\nA delete request with the key and the address of the …\nA get request with the key and the address of the client.\nA gossip request with the message id, writes and the …\nConvenience enum to represent a gossip request with the …\nA nack request with the message id and the address of the …\nA set request with the key, value and the address of the …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreate a <code>ClientRequestWithAddress</code> from a <code>ClientRequest</code> and …\nCreate a <code>GossipRequestWithAddress</code> from a <code>GossipMessage</code> and …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.")