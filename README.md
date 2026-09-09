# BlockRazor Solana Geyser Stream Rust Client

`geyserstream-client-rust` is a Rust example for connecting to the BlockRazor Solana Geyser Stream over gRPC. It demonstrates how to authenticate with an `x-token`, configure Geyser subscription filters, and receive account, transaction, and block updates.

The repository includes an asynchronous Tonic client, Geyser and Solana storage Protocol Buffers definitions, and a build script that generates the required Rust protobuf code.

## Supported subscriptions

The example in `src/example.rs` supports these Geyser Stream subscriptions:

| Subscription | Configuration | Example output |
|---|---|---|
| Accounts | `subscribe_accounts` | Account information |
| Transactions | `subscribe_transactions` | Transaction signature |
| Blocks | `subscribe_blocks` | Block slot |

The default example enables the account subscription and disables the transaction and block subscriptions.

## Requirements

- Rust and Cargo
- Protocol Buffers compiler (`protoc`)
- A BlockRazor authentication token
- Access to the BlockRazor Geyser Stream service

See the [Protocol Buffers installation guide](https://protobuf.dev/installation/#package-manager) for instructions on installing `protoc`.

## Quick start

### 1. Clone the repository

```bash
git clone https://github.com/BlockRazorinc/geyserstream-client-rust.git
cd geyserstream-client-rust
```

### 2. Build the project

```bash
cargo build
```

During the build, `build.rs` compiles:

```text
proto/solana-storage.proto
proto/geyser.proto
```

### 3. Configure the client

Open `src/example.rs` and update the endpoint, token, and subscription settings:

```rust
let endpoint = "https://geyserstream-tokyo.blockrazor.xyz:443";
let token = "";

let subscribe_accounts = true;
let subscribe_transactions = false;
let subscribe_blocks = false;
```

Replace the empty `token` value with your BlockRazor authentication token.

### 4. Run the example

```bash
cargo run --bin example
```

## Subscribe to Solana accounts

Enable account updates with:

```rust
let subscribe_accounts = true;
```

The example subscribes to this account address:

```rust
let subscribe_test_account =
    "HV1KXxWFaSeriyFvXyx48FqG9BoFbfinB8njCJonqP7K".to_string();
```

Replace the address when you need to monitor another account.

The account filter created by the example is:

```rust
SubscribeRequestFilterAccounts {
    account: vec![subscribe_test_account],
    owner: vec![],
    filters: vec![],
    nonempty_txn_signature: Some(true),
}
```

When an account update is received, the example prints the account information:

```rust
println!("receive account: {:?}", msg.account);
```

## Subscribe to Solana transactions

Enable transaction updates with:

```rust
let subscribe_transactions = true;
```

The example creates this transaction filter:

```rust
SubscribeRequestFilterTransactions {
    vote: Some(false),
    failed: Some(false),
    signature: None,
    account_include: vec![],
    account_exclude: vec![],
    account_required: vec![],
}
```

When a transaction update is received, the example converts the signature bytes to a `solana_signature::Signature` and prints the resulting signature:

```rust
println!(
    "receive transaction: {:?}",
    Signature::try_from(tx.signature.as_slice())
        .context("invalid signature")?
        .to_string()
);
```

## Subscribe to Solana blocks

Enable block updates with:

```rust
let subscribe_blocks = true;
```

The example creates this block filter:

```rust
SubscribeRequestFilterBlocks {
    account_include: vec![],
    include_transactions: Some(false),
    include_accounts: Some(false),
    include_entries: Some(false),
}
```

When a block update is received, the example prints its slot:

```rust
println!("receive block: {:?}", msg.slot);
```

## Authentication

The client adds the configured token to the gRPC request metadata as `x-token`:

```rust
request.metadata_mut().insert(
    "x-token",
    token.parse().unwrap(),
);
```

The metadata is added in `to_streaming_request` before the subscription request is sent.

## TLS connection

The example connects to the configured HTTPS endpoint through a Tonic channel:

```rust
let channel = Channel::from_shared(endpoint.to_string())?
    .tls_config(ClientTlsConfig::new().with_native_roots())?
    .connect()
    .await?;
```

TLS is configured with native system certificate roots.

## How the client works

The main flow in `src/example.rs` is:

1. Read the endpoint, token, and subscription settings.
2. Create a TLS-enabled Tonic channel.
3. Create a generated `GeyserClient`.
4. Build account, transaction, and block filter maps.
5. Add the filters to a `SubscribeRequest`.
6. Wrap the request as a stream and attach the `x-token` metadata.
7. Call the bidirectional gRPC `Subscribe` method.
8. Process account, transaction, and block updates from the response stream.
9. Stop reading after the response stream ends or returns an error.

Update variants other than Account, Transaction, and Block are printed as unknown messages by the current example.

## Subscription request

The example creates a `SubscribeRequest` with the configured account, transaction, and block filters:

```rust
let request = SubscribeRequest {
    accounts: accounts_filter,
    transactions: transactions_filter,
    blocks: blocks_filter,
    transactions_status: HashMap::new(),
    entry: HashMap::new(),
    blocks_meta: HashMap::new(),
    slots: HashMap::new(),
    commitment: None,
    accounts_data_slice: vec![],
    ping: None,
    from_slot: None,
};
```

`to_streaming_request` converts this value into the request stream expected by the Geyser `Subscribe` RPC.

## Geyser API definitions

The `Geyser` service in `proto/geyser.proto` defines these RPC methods:

```protobuf
service Geyser {
  rpc Subscribe(stream SubscribeRequest)
      returns (stream SubscribeUpdate) {}
  rpc SubscribeReplayInfo(SubscribeReplayInfoRequest)
      returns (SubscribeReplayInfoResponse) {}
  rpc Ping(PingRequest)
      returns (PongResponse) {}
  rpc GetLatestBlockhash(GetLatestBlockhashRequest)
      returns (GetLatestBlockhashResponse) {}
  rpc GetBlockHeight(GetBlockHeightRequest)
      returns (GetBlockHeightResponse) {}
  rpc GetSlot(GetSlotRequest)
      returns (GetSlotResponse) {}
  rpc IsBlockhashValid(IsBlockhashValidRequest)
      returns (IsBlockhashValidResponse) {}
  rpc GetVersion(GetVersionRequest)
      returns (GetVersionResponse) {}
}
```

The included Rust example calls `Subscribe`. The other RPC methods are defined in the Proto file and generated client, but they are not called by `src/example.rs`.

## Protocol Buffer generation

`build.rs` uses `tonic_prost_build` to generate Rust client code from the Proto files:

```rust
tonic_prost_build::configure()
    .build_server(false)
    .compile_protos(
        &["proto/solana-storage.proto", "proto/geyser.proto"],
        &["proto"],
    )?;
```

Server code generation is disabled because this repository contains a client example.

## Repository structure

```text
.
├── Cargo.toml
├── build.rs                    # Protocol Buffers build configuration
├── README.md
├── proto/
│   ├── geyser.proto            # Geyser service and subscription definitions
│   └── solana-storage.proto    # Solana block and transaction definitions
└── src/
    └── example.rs              # Rust Geyser Stream client example
```

## Main dependencies

| Dependency | Usage in the repository |
|---|---|
| `tonic` | gRPC client and TLS transport |
| `tonic-prost` | Tonic and Prost integration |
| `prost` / `prost-types` | Protocol Buffers messages |
| `tokio` | Asynchronous runtime |
| `tokio-stream` / `futures` | Request and response streams |
| `solana-signature` | Transaction signature conversion |
| `anyhow` | Error context |

## Documentation

See the [BlockRazor Geyser Stream Rust documentation](https://docs.blockrazor.io/streams/block-stream/solana/geyser-stream/rust) for additional service information.

## Frequently asked questions

### What is `geyserstream-client-rust`?

`geyserstream-client-rust` is a Rust example for connecting to the BlockRazor Solana Geyser Stream over gRPC and receiving filtered subscription updates.

### Which Solana updates does the example support?

The example handles account, transaction, and block updates. The included Proto definition contains additional Geyser update types, but the current match statement handles these three variants explicitly.

### How does the Rust client authenticate?

The client inserts the configured BlockRazor token into the `x-token` gRPC metadata field before sending the streaming request.

### Does the client use TLS?

Yes. The example creates a Tonic TLS configuration using native system certificate roots and connects to the configured HTTPS endpoint.

### Which subscription is enabled by default?

The account subscription is enabled by default. Transaction and block subscriptions are disabled until their corresponding Boolean values are changed to `true`.

### What does the example print?

It prints account information for account updates, transaction signatures for transaction updates, and slot numbers for block updates. Other update variants are reported as unknown messages.
