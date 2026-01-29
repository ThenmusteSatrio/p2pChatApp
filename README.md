# P2P Chat Application

A peer-to-peer chat application built as a learning project to explore decentralized networking using libp2p, with a desktop user interface powered by Tauri.

This repository is intended as an educational reference for understanding how peer discovery and direct peer-to-peer communication can be implemented without relying on a centralized chat server.

---

## Introduction

Most modern chat applications depend on centralized servers to route messages between users. While this approach is convenient, it introduces single points of failure and limits flexibility.

This project explores an alternative approach: **peer-to-peer (P2P) communication**, where chat messages are exchanged directly between devices after peers discover each other through a decentralized network.

The focus of this repository is learning and experimentation rather than production readiness.

---

## High-Level Architecture

The system consists of two main components:

1. **Bootstrap Node**
   - Used only as an initial contact point
   - Helps peers join the P2P network
   - Does not relay or store chat messages

2. **Chat Client**
   - Desktop application built with Tauri
   - Contains both the user interface and P2P networking logic
   - Communicates directly with other peers after discovery

Once peers are connected, all chat communication happens **peer-to-peer**, without passing messages through the bootstrap node.

---

## Peer Discovery with Kademlia

Peer discovery in this project is based on **Kademlia Distributed Hash Table (DHT)**, as implemented by libp2p.

Kademlia allows peers to:
- Discover other peers in a decentralized manner
- Store and query peer information without a central registry
- Efficiently locate peers using XOR-based distance metrics

### How Kademlia is used in this project

At a conceptual level, the flow is as follows:

1. A peer connects to a known bootstrap node
2. The peer joins the Kademlia DHT network
3. Peer addresses are propagated through the DHT
4. Peers query the DHT to discover other peers
5. Direct peer-to-peer connections are established

The bootstrap node exists only to help peers **enter the DHT network**.  
After that, peer discovery relies on **Kademlia routing**, not on a central server.

---

## Repository Structure

```
p2pChatApp/
├── vanadinite/   # Bootstrap node (Rust + libp2p + Kademlia)
├── cofe/         # Desktop chat app (Tauri + frontend + libp2p client)
├── LICENSE
└── README.md     # Global documentation
```

Each main folder contains its own README file with more detailed explanations of internal logic and implementation.

---

## Requirements

To run this project, you will need:

- Linux (recommended for networking experiments)
- Rust toolchain
- Node.js and npm
- Basic understanding of networking concepts (IP, ports, peer-to-peer)
- Familiarity with distributed systems is helpful but not required

---

## Installation and Running

### 1. Clone the Repository

```bash
git clone https://github.com/ThenmusteSatrio/p2pChatApp.git
cd p2pChatApp
```

---

### 2. Start the Bootstrap Node

```bash
cd vanadinite
cargo run
```

Leave this process running.

---

### 3. Start the Chat Application

```bash
cd cofe
npm install
npm run tauri dev
```

Run the application on at least two devices or instances to test peer-to-peer communication.

---

## Example Usage

1. Start the bootstrap node
2. Launch the chat application
3. Each client connects to the bootstrap node
4. Peers join the Kademlia DHT
5. Peers discover each other
6. Messages are exchanged directly between peers

No central server is used for chat message delivery.

---

## Learning Focus

This project is useful for learning about:

- Peer-to-peer networking fundamentals
- libp2p architecture and design
- Kademlia DHT and decentralized peer discovery
- Bootstrap-based network initialization
- Rust networking logic
- Desktop application development with Tauri
- Integrating frontend UI with Rust backend logic

---

## Possible Improvements

Some potential directions for further development:

- Improved NAT traversal
- Group chat support
- File transfer between peers
- Better peer identity and reputation management
- Message persistence or offline messaging

---

## Contributing

This project is shared primarily for educational purposes.  
Issues, discussions, and pull requests are welcome.

---

## License

This project is open-source and licensed under the terms specified in the `LICENSE` file.

---

## Notes

This repository is intentionally structured to encourage exploration and experimentation.  
Readers are encouraged to read the code, inspect the Kademlia logic, run the system locally, and modify it to better understand how peer-to-peer systems work.
