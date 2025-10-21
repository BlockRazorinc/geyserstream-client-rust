use {
	anyhow::Context,
    futures::{stream::StreamExt},
    solana_signature::Signature,
    std::{
        collections::{HashMap},
		error::Error,
    },
    tonic::transport::ClientTlsConfig,
	tonic::transport::Channel,
	tonic::IntoStreamingRequest,
    geyser_stream::geyser_client::GeyserClient,
	geyser_stream::subscribe_update::UpdateOneof,
	geyser_stream::SubscribeRequestFilterBlocks,
	geyser_stream::SubscribeRequestFilterTransactions,
	geyser_stream::SubscribeRequest,
	geyser_stream::SubscribeRequestFilterAccounts,
};

mod solana_storage_flat {
    include!(concat!(env!("OUT_DIR"), "/solana.storage.confirmed_block.rs"));
}

pub mod solana {
    pub mod storage {
        pub mod confirmed_block {
            pub use super::super::super::solana_storage_flat::*;
        }
    }
}

pub mod geyser_stream {
    pub use crate::solana;
    include!(concat!(env!("OUT_DIR"), "/geyser.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let endpoint = "https://geyserstream-tokyo.blockrazor.xyz:443";
	let token = "";
	let subscribe_accounts = true;
	let subscribe_transactions = false;
	let subscribe_blocks = false;
	let subscribe_test_account = "HV1KXxWFaSeriyFvXyx48FqG9BoFbfinB8njCJonqP7K".to_string();

	let channel = Channel::from_shared(endpoint.to_string())
		.map_err(|e| Box::<dyn Error>::from(format!("Invalid URI: {}", e)))?
    	.tls_config(ClientTlsConfig::new().with_native_roots())
    	.map_err(|e| Box::<dyn Error>::from(format!("TLS config error: {}", e)))?
		.connect()
		.await
		.map_err(|e| Box::<dyn Error>::from(format!("Connection error: {}", e)))?;

	let mut client = GeyserClient::new(channel.clone());

	// subscribe account filters
	let mut accounts_filter = HashMap::new();
	if subscribe_accounts {
		accounts_filter.insert(
			"client".to_owned(),
			SubscribeRequestFilterAccounts {
				account: vec![subscribe_test_account],
				owner: vec![],
				filters: vec![],
				nonempty_txn_signature: Some(true),
			},
		);
	}

	// subscribe transaction filters
	let mut transactions_filter = HashMap::new();
	if subscribe_transactions {
		transactions_filter.insert(
			"client".to_string(),
			SubscribeRequestFilterTransactions {
				vote: Some(false),
				failed: Some(false),
				signature: None,
				account_include: vec![],
				account_exclude: vec![],
				account_required: vec![],
			},
		);
	}

	// subscribe block filters
	let mut blocks_filter = HashMap::new();
	if subscribe_blocks {
		blocks_filter.insert(
			"client".to_owned(),
			SubscribeRequestFilterBlocks {
				account_include: vec![],
				include_transactions: Some(false),
				include_accounts: Some(false),
				include_entries: Some(false),
			}
		);
	}
	
	// subscribe request
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
    let streaming_req = to_streaming_request(request, token.to_string());

	// subscribe
	let response = client.subscribe(streaming_req).await.unwrap();
	let mut resp_stream = response.into_inner();
    while let Some(message) = resp_stream.next().await {
        match message {
            Ok(msg) => {
                match msg.update_oneof {
                    Some(UpdateOneof::Account(msg)) => {
						println!("receive account: {:?}", msg.account);
                    }
                    Some(UpdateOneof::Transaction(msg)) => {
						let tx = msg
                            .transaction
                            .ok_or(anyhow::anyhow!("no transaction in the message"))?;
						println!("receive transaction: {:?}", Signature::try_from(tx.signature.as_slice()).context("invalid signature")?.to_string());
                    }
                    Some(UpdateOneof::Block(msg)) => {
						println!("receive block: {:?}", msg.slot);
                    }

					// other implementations can be added here

					_ => {
						println!("receive unknown message");
						continue
					}
                }
            }
            Err(error) => {
                println!("stream error: {error:?}");
                break;
            }
        }
    }

    Ok(())
}

fn to_streaming_request(req: SubscribeRequest, token: String) -> impl IntoStreamingRequest<Message = SubscribeRequest> {
    let stream = tokio_stream::iter(std::iter::once(req));
    let mut request = tonic::Request::new(stream);

    request.metadata_mut().insert(
        "x-token",
        token.parse().unwrap(),
    );

    request
}