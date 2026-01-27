# P2P Chat Application

A peer-to-peer chat application built as a learning project to explore decentralized networking using libp2p, with a desktop user interface powered by Tauri.

This repository is intended as an educational reference for understanding how peer discovery and direct peer-to-peer communication can be implemented without relying on a centralized chat server.

---

## Introduction

Most modern chat applications depend on centralized servers to route messages between users. While this approach is convenient, it introduces single points of failure and limits flexibility.

This project explores an alternative approach: **peer-to-peer (P2P) communication**, where chat messages are exchanged directly between devices after peers discover each other.

The focus of this repository is learning and experimentation rather than production readiness.

---

## High-Level Architecture

The system consists of two main components:

1. **Bootstrap Node**
   - Used only as an initial contact point
   - Helps peers discover each other
   - Does not relay or store chat messages

2. **Chat Client**
   - Desktop application built with Tauri
   - Contains both the user interface and P2P networking logic
   - Communicates directly with other peers after discovery

Once peers know each other, all chat communication happens **peer-to-peer**, without passing messages through the bootstrap node.

---

## Repository Structure

```
p2pChatApp/
├── vanadinite/   # Bootstrap node (Rust + libp2p)
├── cofe/         # Desktop chat app (Tauri + frontend + libp2p client)
├── LICENSE
└── README.md     # Global documentation
```

---

## Requirements

To run this project, you will need:

- Linux (recommended for networking experiments)
- Rust toolchain
- Node.js and npm
- Basic understanding of networking concepts (IP, ports, peer-to-peer)

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

Run the application on at least two devices or instances.

---

## Example Usage

1. Start the bootstrap node
2. Launch the chat application
3. Each client connects to the bootstrap node
4. Peers discover each other
5. Messages are exchanged directly between peers

No central server is used for chat messages.

---

## Learning Focus

This project is useful for learning about:

- Peer-to-peer networking fundamentals
- libp2p concepts and architecture
- Bootstrap-based peer discovery
- Rust networking logic
- Desktop application development with Tauri
- Integration between frontend UI and Rust backend

---

## Possible Improvements

- End-to-end encryption
- Improved NAT traversal
- Group chat support
- File transfer between peers
- Better peer identity management

---

## Contributing

This project is shared primarily for educational purposes.  
Issues and pull requests are welcome.

---

## License

This project is open-source and licensed under the terms specified in the `LICENSE` file.

---

## Notes

This repository is intentionally structured to encourage exploration and experimentation.  
Readers are encouraged to read the code, run it locally, and modify it to better understand peer-to-peer systems.
