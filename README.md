# Motivation
Full-scale Danksharding requires a lot of small pieces of data (samples), and attestations to said data, to be communicated efficiently across Ethereum's Consensus Layer p2p network.  

Design questions around *how* this new information should be communicated is still up in the air and is known as the Data Availability Sampling Networking problem.  Danny wrote a [Request For Proposals](https://github.com/ethereum/requests-for-proposals/blob/master/open-rfps/das.md) post which provides background to learn more about the problem.

# Summary
Within DAS Playground I implement the basic p2p network construction behind one possible solution: a Secure Kademlia DHT discv5 overlay (validators only routing table).  See [Dankrad's Post](https://notes.ethereum.org/@dankrad/S-Kademlia-DAS) for details.

The idea behind this repository is to build out the DAS p2p networking stack needed to create a Secure K-DHT, then integrate the secure overlay protocol within Timofey and Eric's DAS networking simulation, [DAS Prototype](https://github.com/ChainSafe/das-prototype).

### Goals:
1.  Spin up DASNodes that instantiate: 
    - Discovery Protocol                       [X]
    - Overlay Protocol                         [X]
        1. Main DAS Network (*Subprotocol*)    [ ]     
        2. Secure DAS Network (*Subprotocol*)  [ ]     
2.  Manipulate node's state through our main and secure DAS networks  
3.  Add peers to their routing tables and samples to data stores through overlay requests.

### To Do:
1. Instantiate message processing within each node.  
   *Currently figuring out logging for when messages are being processed*
2. Send message via overlay protocol
3. Modify create_overlay() to be generalizable for creating a Secure Overlay

### Note:
I'm trying to design this repository to be easy to comprehend (at the cost of efficiency), making these networking concepts more accessible.

If you've got suggestions for cleaner code or have any questions, make a PR or reach out!  