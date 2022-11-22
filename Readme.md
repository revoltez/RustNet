<p align="center">
<img src="./Bissmillah.svg" class="center"></p>
</p>

# rust_net

A Reactive Distributed Systems ToolBox, it provides an easy modular approach to architect your Distributed application.


> __Warning__:
*this project is for learning purposes only, especially for beginners that want to implement a simple network in a modular approach, it should not be used in production, instead check [libp2p](https://libp2p.io/) and its rust implementation*
# charachteristics

- Has minimum dependancies to third party libraries
- Loosely coupled components
- Extensible

# install

Add this to your cargo.toml file

```
rust_net = "1.0.0"
```

# Components
Components are threads that communicate by sending messages to each other via ```flume``` channels(very similar to mpsc channels), and by also publishing and subscribing to certain messages(like broadcasting messages to whomever is interetsed).

every component must implement the ```NetComponent``` trait so it could be easily hooked with other components.

## Node

- a mediator between component, for a fast and easy way to get started with rust_net components
- add your own components without worrying about the implementation details of communicating with other components
- Node acts as a mediator between components in that it receives external calls and forward them to the appropriate Components
- can register user defined callbacks upon receival of external Messages

PS: Order of addition of Components mut be respected in order for Node to work correctly, For example every Reliable Delivery Component Requires a Faillure detector, which means messing up the order will cause node to panic before execution.

## Faillure Detector

- assumes synchrony which means timing Bounds of network and processes are fixed with the `timeout` variable

## Delivery

- BestEffor Delivery

     - Gurantees reliability only if sender is correct

- Reliable Delivery

     - gurantees reliability independent of whether the sender is correct or not
     - it ensures all or none of the correct nodes gets the message
     - even if the sender crashes without delivering the message to everyone, every correct node that already received the message will play the role of the sender
     - Requires a Faillure detector

Example: need to guarantee that a message is reached to all correct nodes even if if the sender crashes after sending to one correct node.

- Uniform Reliable Delivery

     - gurantees reliability and also considers the behaviour of failed nodes,
     - reliable Delivery is faster but it doesnt gurantee if all nodes delivered the message including Failled nodes

## Example

- create a Node that has a Faillure detector,Commits Messages only if all nodes Commit, has 5 maximum peers and triggers a user defined callback whenever a Uniform Reliable Delivery message is received

```
let peers = vec![

        String::from("172.17.0.2:8888"),
        String::from("172.17.0.3:8888"),
        String::from("172.17.0.4:8888"),
        String::from("172.17.0.5:8888"),
    ]
let mut node = Node::new("MyNode".to_string(), 8888, 5);

node.has_faillure_detector(Duration::from_millis(5000), Duration::from_millis(5000))
    .on_receive_message(
            MessageType::UniformReliableDelivery,
            Box::new(|msg| println!("FROM USER: Triggering my callback")),
        )
    .has_uniform_reliable_delivery();

node.start().join().expect("Failled Joining node handle");
```

- if the Faillure Detector detects a crash it will Publish an event and the concerned components would React to this event accordignly, in the case of Uniform Reliable Broadcast it will redistributed all previous crashed node messages to gurantee Reliability

To Run the example open a terminal and type the following:

```
./Build.sh
./Run.sh
```

this will create a mesh network with 4 nodes, each node runing in a seperate docker container:

https://user-images.githubusercontent.com/24751547/166156115-513458b9-7c33-4fb6-b850-ea50e4a48212.mp4

# TODO
- [ ] Add Lamport Logical Clock component
- [ ] Add Vector clocks component
- [ ] Add causal delivery component

 
# Contact

email : salih.houadef@gmail.com

linkedin : https://www.linkedin.com/in/houadef-salih
