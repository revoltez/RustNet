<p align="center">
<img src="./Bissmillah.svg" height="100px" width="300px" class="center"></p>
</p>

# rust_net
</p>
A Reactive Distributed Systems ToolBox, it provides an easy modular approach to architect your Distributed application.



##
> __Warning__:
*this project is for learning purposes only, it should not be used in production, instead check [libp2p](https://libp2p.io/) and its rust implementation*
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
An intermediary among components, providing a swift and straightforward approach to initiate rust_net components. Integrate your own components seamlessly, free from concerns about the underlying intricacies of inter-component communication. The Node serves as an intermediary, receiving external calls and effectively relaying them to the designated components. It's also possible to register custom user callbacks upon the receipt of external messages.

Note: The sequence of adding components must be upheld for the Node to function accurately. ensuring the correct order is vital, 
for instance  each Reliable Delivery Component is dependent on a Failure Detector. Disrupting this order might lead to the Node panicking even before execution begins.

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
