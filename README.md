# Merkle tree

This is an implementation of algorithmic structure called "Merkle tree".

In various forms it lies at the core of blockchains, some torrent extensions
and distributed databases like Cassandra.\
Hashing sets of blocks with Merkle tree allows to verify blocks received from a perr more efficiently (using Merkle proof).

# Where to start?

Glance at the torrent-like scenario at src/lib.rs:126\
The algorithms itself implemented in src/merkle.rs
