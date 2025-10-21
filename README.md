# GeyserStream-client-rust
example for Geyser Stream in Rust

# Document
see [document](https://blockrazor.gitbook.io/blockrazor/solana/geyser-stream/rust)

# Quickstart

1. **Download git repository**

   `git clone https://github.com/BlockRazorinc/geyserstream-client-rust.git`

2. **Change directory**

   `cd geyserstream-client-rust`

3. **Download dependencies**

   install [protoc](https://protobuf.dev/installation/#package-manager), then `cargo build`

4. **Edit src/example.rs**

	```
	let endpoint = "https://geyserstream-tokyo.blockrazor.xyz:443";
	let token = "";
	let subscribe_accounts = true;
	let subscribe_transactions = false;
	let subscribe_blocks = false;
	```

- To subscribe to block information, set subscribe_blocks to true.

- To subscribe to account information, set subscribe_accounts to true.

- To subscribe to transaction information, set subscribe_transactions to true.
  
5. **Run example**
   
   `cargo run --bin example`
