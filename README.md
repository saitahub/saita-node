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