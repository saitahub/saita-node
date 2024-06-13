<h1></p>SaitaBlockChain(SBC)</code></h1>

SaitaBlockChain(SBC) is a next-generation blockchain innovation. It is a L0 chain that allows the addition of parachains to the blockchain. It is also compatible with Ethereum as it has EVM (Ethereum Virtual Machine) integrated.

<p align="center">
  <img src="/docs/SBC.png">
</p>

# Table of Contents
* Description
* Features
* Getting Started
* Rust Setup
* Build and Run


# Description
SaitaBlockChain(SBC) is a next-generation blockchain innovation that enables the addition of parachains to the blockchain. It offers compatibility with Ethereum through the integration of EVM. This blockchain solution provides advanced features and aims to revolutionize the decentralized ecosystem.


# Features
SaitaBlockChain(SBC) incorporates the following features:

- Consensus related pallets: Babe & GRANDPA.
- Staking related pallets: staking, session, authorship, im-online, offences, utility.
- Governance related pallets: collective, membership, elections-phragmen, democracy, treasure
- xcm

- These components form the minimum required setup to start a NPoS (Nominated Proof-of-Stake) testnet.

# Getting Started
Follow the steps below to get started with SBC: 

### Rust Setup
Before building and running SaitaBlockChain(SBC), ensure you have completed the [Dev Docs](https://docs.substrate.io/install/) Installation. This will set up the necessary Rust environment.

### Build and Run
To build the node and run it after a successful build, use the following command:

```sh
cargo build --release
./target/release/Saita-Chain --dev --tmp
```

# Node Setup Guide
## Type of nodes
Saitachain provides three types of node options to user for connecting with the
saitachain network.

### Full Node
Full nodes act as a server in a decentralized network. They also maintain a copy of the blockchain. Full nodes enable you to read the current state of the chain and to submit extrinsic on the network without depending on a centralized infrastructure provider.

### Validator Node
Validators are responsible for the network's infrastructure and upkeep. Validators are in charge of creating new blocks, as well as ensuring the network's finality and, ultimately, its security. Validators job is demanding as they need to run costly operations. Validator must also stake their saitachain coin (STC), a native token of saitachain blockchain, as a guarantee of good conduct, and whenever validator make mistake, their stake is slashed. In contrast, when they follow the rules, they are generously compensated.

Running a validator on a live network is a lot of responsibility! validator will be responsible not just for their own stake, but also for the stakes of their existing nominators. Validators money and reputation will be jeopardized if you make a mistake, their stake, including the nominators will get slashed.

### Archive Node
Archive nodes are similar to full nodes except that they store all past blocks with complete state available for every block. Archive nodes are most often used by utilities—such as block explorers, wallets, discussion forums, and similar applications—that need access to historical information.

## System requirements to run a saitachain node
### Minimum Requirements
- CPU with 4+ cores
- 8 GB RAM
- 100 GB SSD

### Recommended Requirements
- CPU with 8+ cores
- 32 GB RAM
- 100 GB SSD
- Ubuntu Version : 20.04.0

### Best Practices
- Mount extra volume for blockchain storage
- Assign elastic IP to instance
- Alarm on blockchain process and extra volume
- Allow only minimum ports
  - 30333 : For Blockchain discovery
  - 9944 : For web sockets and rpc calls
  - 22 : For ssh 

## Installation Process
The following steps mentioned below are to be followed for node installation using docker on a cloud server.

Note: You need to use Docker to run your code. Since this process is a bit advanced, you need to be familiar with Docker and node setup experience using Docker. Also, when you run a saitachain node, the process only listens to the local host by default.

<p> Step 1: Log in to the cloud server (AWS | AZURE | GCP etc) </p>
<p> Step 2: Open the terminal </p>
<p> Step 3: Download the Docker image using the following command: </p> 

```
sudo docker pull saitachain/sbc-mainnet-node:latest
```
<p> Step 4: Run the dockersied RPC Command: </p>

<p>Full Node Command:</p>

```
sudo docker run -d -p 9944:9944 -p 30333:30333 -v /external_volume/volume_name:/data saitachain/sbc-mainnet-node:latest --base-path /tmp/user_input --port 30333 --rpc-port 9944 --no-telemetry --rpc-methods Unsafe --rpc-external --name user_input --rpc-cors all
```

<p> Validator Node Command: </p>

```
sudo docker run -d -p 9944:9944 -p 30333:30333 -v /external_volume/volume_name:/data saitachain/sbc-mainnet-node:latest --base-path /tmp/user_input --port 30333 --rpc-port 9944 --no-telemetry --validator --rpc-methods Unsafe --rpc-external --name user_input --rpc-cors all --discover-local
```

<p> Archive Node Command: </p>

```
sudo docker run -d -p 9944:9944 -p 30333:30333 -v /external_volume/volume_name:/data saitachain/sbc-mainnet-node:latest --base-path /tmp/user_input --port 30333 --rpc-port 9944 --no-telemetry --rpc-methods Unsafe --rpc-external --name user_input --rpc-cors all --pruning archive --discover-local
```

### Note: Users must change user_input, volume_name and external_volume before running the node 
- user_input : Enter your desired node name in place of user_input in the above command, else the node will run with “user_input” name.
- Volume_name :Enter the name of the volume where you want to make your node database.
- External_volume: volume that you will attach to your instance. The node is syncing with the blockchain network and it would take some time to complete the syncing.
