/*
    Trin and E&T differ with how talkreq/resp messages are handled.  
    
    Trin deals with multiple overlay protocols simultaneously, while DAS Prototype only has to manage one overlay. So...
        DAS Prototype can spawn one task (per node) to handle all client side message passing to another node,
        
        Trin made a proxy that sits between Discv5's talkreq/talkresp and spawned tasks each containing handlers with 
        overlay-specific req handling logic.
    
    
    
    Draw some inspiration from Trin since I'll be dealing with multiple overlays
    
    Trin:  https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174
    DAS Prototype: https://github.com/ChainSafe/das-prototype/blob/main/src/main.rs#L261
*/


const SECURE_DAS_PROTOCOL_ID: &str = "SECURE_DAS";