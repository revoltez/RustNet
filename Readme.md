<p align="center">
<img src="./Bissmillah.svg" class="center"></p>
</p>

# RustNet

A Fast and Reactive Distributed Systems ToolBox

# charachteristics

- Simplistic and Clean Architecture (builder/mediator)
- unopinianated in that it has minimum dependancies to third party libraries
- loosely coupled components
- extensible

# install
Add this to your cargo.toml file
```
rust_net = "0.1.1"
```

# Components

## Node

- abstracts all the complexity of Rustnet in few customizable Builder statements
- add your own components with one line and without worrying about the implementation details of communicating with other components
- Node acts as a mediator between components in that it receives external calls and forward them to the appropriate Components
- can register user defined callbacks upon receival of external Messages

PS: Order of Components mut be respected in order for Node to work correctly, For example the all Reliable Delivery Components Requires a Faillure detector

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
  - example: need to guarantee that a message is reached to all correct nodes even if if the sender crashes after sending to one correct node.
- uniform Reliable Delivery
  - gurantees reliability and also considers the behaviour of failed nodes,
  - reliable Delivery is faster but it doesnt gurantee if all nodes delivered the message including Failled nodes

# Example

- create a Node that has a Faillure detector,Commits Messages only if all nodes Commit, has 5 maximum peers and triggers a user defined callback whenever a Uniform Reliable Deliver message is received

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

# Contact
email : salih.houadef@univ-constantine2.dz

linkedin : https://www.linkedin.com/in/houadef-salih-2b92a0188

