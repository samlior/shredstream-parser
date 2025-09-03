use crate::transaction::common::{format_sol_amount, format_token_amount, WSOL_MINT};
use borsh::BorshDeserialize;
use solana_sdk::{instruction::CompiledInstruction, transaction::VersionedTransaction};
use transaction_protos::transaction::Transaction;

pub const AZCZ_PROGRAM_ID: &str = "AzcZqCRUQgKEg5FTAgY7JacATABEYCEfMbjXEzspLYFB";

const BUY_TOKENS_IX: u8 = 11;

#[derive(BorshDeserialize, Debug)]
struct BuyArgs {
    sol_amount: u64,
    token_amount: u64,
}

fn parse_buy_tokens_args(
    transaction: &VersionedTransaction,
    instruction: &CompiledInstruction,
) -> Result<Transaction, &'static str> {
    if instruction.accounts.len() < 8 {
        return Err("accounts too short");
    }

    let static_keys = transaction.message.static_account_keys();

    let mint_index = instruction.accounts[3] as usize;
    if mint_index >= static_keys.len() {
        return Err("mint index out of bounds");
    }
    let mint = static_keys[mint_index];

    let user_index = instruction.accounts[7] as usize;
    if user_index >= static_keys.len() {
        return Err("user index out of bounds");
    }
    let user = static_keys[user_index];

    if let Ok(args) = BuyArgs::try_from_slice(&instruction.data[1..]) {
        Ok(Transaction {
            tx_hash: transaction.signatures[0].to_string(),
            maker: user.to_string(),
            token0_address: mint.to_string(),
            token1_address: WSOL_MINT.to_string(),
            token0_amount: format_token_amount(args.token_amount),
            token1_amount: format_sol_amount(args.sol_amount),
            program: "pump".to_string(),
            event: "buy".to_string(),
        })
    } else {
        Err("failed to parse buy tokens args")
    }
}

pub fn parse_azcz_instruction(
    transaction: &VersionedTransaction,
    instruction_index: usize,
) -> Option<Result<Transaction, &'static str>> {
    let message = &transaction.message;
    let instructions = message.instructions();

    if instruction_index >= instructions.len() {
        return Some(Err("instruction index out of bounds"));
    }

    // Parse instruction type
    let instruction = &instructions[instruction_index];
    if instruction.data.is_empty() {
        return Some(Err("data is empty"));
    }

    let discriminator = instruction.data[0];

    match discriminator {
        BUY_TOKENS_IX => {
            return Some(parse_buy_tokens_args(transaction, instruction));
        }
        _ => None,
    }
}
