use log::warn;
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use transaction_protos::transaction::Transaction;

mod common;

mod pump_parser;
use pump_parser::{parse_pump_instruction, PUMP_PROGRAM_ID};

mod azcz_parser;
use azcz_parser::{parse_azcz_instruction, AZCZ_PROGRAM_ID};

mod f5tf_parser;
use f5tf_parser::{parse_f5tf_instruction, F5TF_PROGRAM_ID};

pub fn parse_transaction(transaction: &VersionedTransaction) -> Vec<Transaction> {
    let mut transactions = Vec::new();

    let tx_hash = transaction.signatures[0].to_string();
    let static_keys = transaction.message.static_account_keys();
    let instructions = transaction.message.instructions();

    for instruction_index in 0..instructions.len() {
        let instruction = &instructions[instruction_index];
        if instruction.program_id_index as usize >= static_keys.len() {
            // ignore
            continue;
        }

        let program_id = static_keys[instruction.program_id_index as usize];
        if program_id == Pubkey::from_str_const(PUMP_PROGRAM_ID) {
            let result = parse_pump_instruction(transaction, instruction_index);
            match result {
                Some(Ok(transaction)) => {
                    transactions.push(transaction);
                }
                Some(Err(e)) => {
                    warn!(
                        "failed to parse pump instruction: {:?}, tx_hash: {}",
                        e, tx_hash
                    );
                }
                _ => {}
            }
        } else if program_id == Pubkey::from_str_const(AZCZ_PROGRAM_ID) {
            let result = parse_azcz_instruction(transaction, instruction_index);
            match result {
                Some(Ok(transaction)) => {
                    transactions.push(transaction);
                }
                Some(Err(e)) => {
                    warn!(
                        "failed to parse azcz instruction: {:?}, tx_hash: {}",
                        e, tx_hash
                    );
                }
                _ => {}
            }
        } else if program_id == Pubkey::from_str_const(F5TF_PROGRAM_ID) {
            let result = parse_f5tf_instruction(transaction, instruction_index);
            match result {
                Some(Ok(transaction)) => {
                    transactions.push(transaction);
                }
                Some(Err(e)) => {
                    warn!(
                        "failed to parse f5tf instruction: {:?}, tx_hash: {}",
                        e, tx_hash
                    );
                }
                _ => {}
            }
        }
    }

    transactions
}
