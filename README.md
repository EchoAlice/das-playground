The idea behind this repository is to play with + showcase concepts that are used within [DAS Prototype](https://github.com/ChainSafe/das-prototype).

I'm redesigning codebase to be more friendly to creating a plethera of module like projects that can be run independently within DAS Playground.


### Goals:
1.  Spin up DASNodes that instantiate the Discovery Protocol.   
2.  Add information to their routing tables! And their data stores.
3.  Complete new DASNodes
    - DASNode
        1. Discovery Protocol
        2. Libp2p Service
        3. Samples
        4. Handled_ids
        5. Overlay Protocol 
4.  Create new routing table
5.  Create secure kademlia routing table


### Currently:
1. Restructuring repo.  main.rs will be used to execute different files 
2. Accessing specific information from a node's discv5 server.  Look into calling a server's api!