<div align="center">
    <img src="./logo.png" height="150" width="150" />
<h1 align="center">
    Rustic Chain of Blocks
</h1>

> Rustic Chain of Blocks is a minimalistic blockchain implementation with P2P networking and HTTP-based RPC-like server functionalities. This project was built with the following [specifications](https://gist.github.com/manav2401/2bea3e1b15efea4f34f6516a3841c6b0) as a reference.

</div>

## Technologies used in the project:

- [rust-libp2p](https://github.com/libp2p/rust-libp2p)
- [ethers-rs](https://github.com/gakonst/ethers-rs)
- [rlp](https://github.com/alloy-rs/rlp)

## Usage

A prerequisite for this project is to have Rust and Cargo installed. Here's [an installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

> [!WARNING]
> If you start the blockchain node first without any P2P nodes running, it will panic after some time, raising an error saying `InsufficientPeers`.

Firstly, start a P2P node using the following:

```
cargo run -p rustic-chain-of-blocks --bin p2p
```

This will start a P2P node, which will vote `Yes/No` on the proposed blocks received by the blockchain node. You can start as many P2P nodes as you wish. Just run the above command in different terminals.

Then, you can start the blockchain node using the following:

```
cargo run -p rustic-chain-of-blocks --bin node
```

In your work directory, this will create 3 json files, namely `accounts.json`, `blockchain.json`, and `mempool.json`. These files store the state of the blockchain as it progresses.

- `accounts.json` stores all the account states of the blockchain.
- `blockchain.json` stores all the blocks that are produced in the blockchain.
- `mempool.json` temporarily stores the transaction you send via the `tx.rs` bin or `http_server`.

To send a transaction, execute this command in another terminal:

```
cargo run -p rustic-chain-of-blocks --bin tx
```

To start the HTTP server, execute the following commands in another terminal:

```
cd http_server
```

```
npm run server
```

Below are the endpoints of the server along with their functionalities:

POST

- `/sendTx?from={address}&to={address}&value={amount}&pk={privateKey}`

GET

- `/blockNumber`: Returns the most recent block number.
- `/block?number={number}`: Given the block number, returns the contents of a block.
- `/block?hash={hash}`: Given the block hash, returns the contents of a block.
- `/tx?hash={hash}`: Given the transaction hash, returns the contents of a transaction.
- `/getNonce?address={address}`: Given the address, returns the current nonce of that account.
- `/getBalance?address={address}`: Given the address, returns that account's current balance.
