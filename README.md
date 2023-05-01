# Motivation
As per Ethereum's [rollup-centric roadmap](https://ethereum-magicians.org/t/a-rollup-centric-ethereum-roadmap/4698), full-scale Danksharding requires extra *blob* data to be [available](https://github.com/ethereum/research/wiki/A-note-on-data-availability-and-erasure-coding) for others to download.  Efficient verification of a blob's availability can be achieved through the process of [data availability sampling](https://hackmd.io/@vbuterin/sharding_proposal#ELI5-data-availability-sampling), allowing the network to come to consensus on this data's availability without increasing the computational resources required for a node.

Between this increase in information (from 90 KB to 128 MB per block!) and our new communication paradigm (data availability sampling), Ethereum's Consensus Layer P2P Network needs an upgrade.

Designs around *how* Ethereum's Consensus Layer P2P Network network can communicate this new information is an open question known as the [Data Availability Sampling Networking problem](https://github.com/ethereum/requests-for-proposals/blob/master/open-rfps/das.md).



# Summary
One possible solution to the DAS Networking problem is to create a **Secure Kademlia DHT overlay network atop Discv5**.

DAS Playground combines Dankrad's idea of a [Secure Kademlia DHT]((https://notes.ethereum.org/@dankrad/S-Kademlia-DAS)) with the main data struct found within Timofey and Eric's discv5 overlay simulation, a [DASNode](https://github.com/ChainSafe/das-prototype/blob/main/src/main.rs#L88) within [DAS Playground](https://github.com/ChainSafe/das-prototype), to create the networking stack nodes need for this p2p networking design.

**This repository was created to facilitate understanding of these overlay protocols to integrate this secure overlay network within a forked version of Timofey and Eric's simulation, [Model DAS](https://github.com/EchoAlice/Model-DAS).**



&nbsp;
![dasnode_image](./assets/DASNode.png)

*** DAS Prototype and Model DAS leverage Trin's [overlay protocol](https://github.com/ethereum/trin/tree/master/trin-core/src/portalnet) + Sigma Prime's [discv5 protocol](https://github.com/sigp/discv5) to support these custom overlay networks. 


&nbsp;
---
### Done
1. Created DASNodes with protocol structs, services and message processing to instantiate all three networks
2. Nodes can send and receive messages via overlay subnetworks


### To Do
1. Send all message types via overlay protocols
2. Remove Duplicate code:
    - Overlay functions
3. Integrate SecureDAS Overlay within [Model DAS](https://github.com/EchoAlice/Model-DAS) ???
4. Create attacks and fallback network logic ???