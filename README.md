The idea behind this repository is to play with + showcase concepts that are used within [DAS Prototype](https://github.com/ChainSafe/das-prototype).  Along with DAS-Prototype, I'm also utilizing Brechy's [CL-P2P-Setup](https://github.com/brech1/cl-p2p-setup) as a resource to implement these Consensus Layer P2P concepts that may (down the line) be used for Data Availability Sampling.

The repo is now friendly to creating a plethera of module like projects that can be run independently within DAS Playground!


### Goals:
1.  Spin up DASNodes that instantiate: 
    1. Discovery Protocol
    2. Overlay Protocol 
    3. Libp2p Service
    4. Samples
    5. Handled_ids
2.  Add information to their routing tables! And their data stores.      
3.  Add a secure kademlia overlay to our DASNode data structure 


### Currently:
1.  Setting discv5's network topology (placing peers in discv5's routing table)

### To Do:
1. Create event loop for each discv5 server
2. Instantiate overlay protocol
3. Learn how to manipulate shared state of complex data structures via 
3. Instantiate libp2p service 