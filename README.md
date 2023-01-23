# Motivation
Full-scale Danksharding requires a lot of small pieces of data (samples), and attestations to said data, to be communicated efficiently across Ethereum's Consensus Layer p2p network.  

Design questions around *how* this new information should be communicated is still up in the air and is known as the Data Availability Sampling Networking problem.  Danny wrote a [Request For Proposals](https://github.com/ethereum/requests-for-proposals/blob/master/open-rfps/das.md) post which provides background to learn more about the problem.

# Summary
Within DAS Playground I implement the basics behind one possible solution: a Secure Kademlia DHT discv5 overlay (validators only routing table).  See [Dankrad's Post](https://notes.ethereum.org/@dankrad/S-Kademlia-DAS) for details.

The idea behind this repository is to build out the DAS p2p networking stack needed to create a Secure K-DHT, then integrate the secure overlay protocol within Timofey and Eric's DAS networking simulation, [DAS Prototype](https://github.com/ChainSafe/das-prototype).

### Goals:
1.  Spin up DASNodes that instantiate: 
    - Discovery Protocol        [X]
    - Overlay Protocol          [X]
    - Secure Overlay Protocol   [ ] 
2.  Add information to their routing tables and data stores.

### To Do:
1. Send messages via overlay protocol
2. Instantiate message processing within each node
3. Modify create_overlay() to be generalizable for creating a Secure Overlay