The idea behind this repository is to build p2p networking concepts used within Timofey and Eric's DAS networking simulation, [DAS Prototype](https://github.com/ChainSafe/das-prototype).
Once the DAS Networking stack has been implemented, I'll integrate the secure overlay protocol within DAS Prototype.

### Goals:
1.  Spin up DASNodes that instantiate: 
    - Discovery Protocol
    - Overlay Protocol 
    - Secure Overlay Protocol (validators only routing table)
2.  Add information to their routing tables and data stores.


### Currently:
1.  Instantiating the overlay protocol 

### To Do:
1. Create event stream for each discv5 server
2. Instantiate overlay protocol
3. Send messages via overlay protocol