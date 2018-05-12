# Merkle tree

This is one of implementations of structure Merkle tree.
In various forms it is the base of blockchains, some torrent extensions
and databases like Cassandra. Hashing sets of block with Merkle tree
allows to more efficiently verify blocks coming from a peer
(using Merkle proof).

# Where to start?

Glance at the torrent-like scenario at src/lib.rs:126

The algorithms itself implemented in src/merkle.rs
