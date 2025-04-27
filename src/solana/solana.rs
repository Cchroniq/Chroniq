use std::{error::Error, str::FromStr};

use log::error;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction, UiTransactionEncoding, option_serializer::OptionSerializer,
};

use crate::conf::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedTransaction {
    pub signature: String,
    pub slot: Option<u64>,
    pub block_time: Option<i64>,
    pub program_id: Option<String>,
    pub success: bool,
    pub logs: Vec<String>,
    pub encoding_type: String,
    pub raw_data: Option<serde_json::Value>,
    pub from: String,
    pub to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleContent {
    pub p: String,
    pub uri: String,
    pub title: String,
    pub content: String,
}

fn parse_raw_data(transcation: &ProcessedTransaction) -> anyhow::Result<TitleContent> {
    if !transcation.success {
        return Err(anyhow::Error::msg("transcation success vale has false"));
    }
    if let Some(raw_data) = &transcation.raw_data {
        let parsed_value = match raw_data {
            serde_json::Value::String(s) => serde_json::from_str(s).map_err(|e| {
                anyhow::Error::msg(format!("failed to parse raw_data string: {}", e))
            })?,
            _ => raw_data.clone(),
        };
        match serde_json::from_value::<TitleContent>(parsed_value) {
            Ok(chroniq) => {
                if chroniq.p != "CHRO" {}
                if chroniq.uri.trim().is_empty()
                    || chroniq.title.trim().is_empty()
                    || chroniq.content.trim().is_empty()
                {
                    error!("Invalid chroniq data: {:?}", chroniq);
                    return Err(anyhow::Error::msg("Invalid chroniq data"));
                }
                return anyhow::Ok(chroniq);
            }
            Err(e) => {
                error!("{:?}", e);
                return Err(anyhow::Error::msg(format!(
                    "raw_data parse obj failed: {}",
                    e
                )));
            }
        }
    } else {
        return Err(anyhow::Error::msg("raw_data has None"));
    }
}

#[allow(dead_code)]
fn get_random_point(solana_points: &Vec<String>) -> &String {
    let mut rng = rand::rng();
    solana_points.choose(&mut rng).unwrap()
}

#[allow(dead_code)]
pub async fn solana_http_query(
    tx_hash: &str,
    config: &Config,
) -> anyhow::Result<(), anyhow::Error> {
    let solana_point = get_random_point(&config.solana_points);
    match signature_query(solana_point, tx_hash).await {
        Ok(transaction) => {
            match parse_raw_data(&transaction) {
                Ok(_title_content) => {
                    // let author = &transaction.from;
                    // let image_url = &title_content.uri;
                    // let title = &title_content.title;
                    // let content = &title_content.content;
                    //download img& upload to AWS OSS
                    anyhow::Ok(())
                }
                Err(_e) => {
                    error!("raw_data parse json failed Ignore this Errors");
                    return anyhow::Ok(());
                }
            }
        }
        Err(e) => Err(anyhow::Error::msg(format!("signature_query err : {:?}", e))),
    }
}

async fn signature_query(rpc_url: &str, signature: &str) -> anyhow::Result<ProcessedTransaction> {
    let signature = Signature::from_str(signature)?;
    let client = RpcClient::new(rpc_url.to_string());
    let transaction_result = client
        .get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await;
    match transaction_result {
        Ok(transaction) => match extract_transaction_info(transaction, &signature).await {
            Ok(transaction_info) => anyhow::Ok(transaction_info),
            Err(e) => Err(anyhow::Error::msg(format!("covernt failed: {:?}", e))),
        },
        Err(e) => {
            error!("extract_transaction_info err: {}", e);
            error!("please check  RPC URL and network");
            Err(anyhow::Error::msg(format!(
                "extract_transaction_info: {:?}",
                e
            )))
        }
    }
}

async fn extract_transaction_info(
    transaction: EncodedConfirmedTransactionWithStatusMeta,
    signature: &Signature,
) -> Result<ProcessedTransaction, Box<dyn Error>> {
    let mut processed_tx = ProcessedTransaction {
        signature: signature.to_string(),
        slot: Some(transaction.slot),
        block_time: None,
        from: "".to_string(),
        to: Vec::new(),
        program_id: None,
        success: false,
        logs: Vec::new(),
        encoding_type: String::new(),
        raw_data: None,
    };
    if let Some(meta) = transaction.transaction.meta {
        processed_tx.success = meta.status.is_ok();
        if let OptionSerializer::Some(logs) = meta.log_messages {
            processed_tx.logs = logs;
        }
    };
    match &transaction.transaction.transaction {
        EncodedTransaction::Json(ui_transaction) => {
            processed_tx.encoding_type = "Json".to_string();
            match &ui_transaction.message {
                UiMessage::Parsed(parsed_message) => {
                    if !&parsed_message.account_keys.is_empty() {
                        for account in &parsed_message.account_keys {
                            if account.signer {
                                processed_tx.from = account.pubkey.clone()
                            } else {
                                if account.writable {
                                    processed_tx.to.push(account.pubkey.clone());
                                }
                            }
                        }
                    }
                    if !&parsed_message.instructions.is_empty() {
                        if parsed_message.instructions.len() > 2 as usize {
                            for instruction in &parsed_message.instructions {
                                match &instruction {
                                    UiInstruction::Parsed(UiParsedInstruction::Parsed(
                                        parsed_info,
                                    )) => {
                                        if parsed_info.program == "spl-memo".to_string()
                                            && parsed_info.program_id
                                                == "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
                                                    .to_string()
                                        {
                                            let data = parsed_info.parsed.clone();
                                            processed_tx.program_id =
                                                Some(parsed_info.program_id.clone());
                                            processed_tx.slot = Some(transaction.slot);
                                            processed_tx.block_time = transaction.block_time;
                                            processed_tx.raw_data = Some(data);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(processed_tx)
}
