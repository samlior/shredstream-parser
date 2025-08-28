use borsh::BorshDeserialize;
use solana_sdk::{instruction::CompiledInstruction, transaction::VersionedTransaction};
use transaction_protos::transaction::Transaction;

// reference: https://github.com/dev12375/Jito-Shredstream-Client/blob/main/src/transaction/pump_parser.rs

pub const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

// Pump program ID
pub const PUMP_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

// Pump instruction number constants
pub const CREATE_COIN_IX: u8 = 24;
pub const EXTENDED_SELL_IX: u8 = 51;
pub const BUY_TOKENS_IX: u8 = 102;

#[derive(BorshDeserialize, Debug)]
struct ExtendedSellArgs {
    amount: u64,
    min_sol_output: u64,
}

#[derive(BorshDeserialize, Debug)]
struct BuyArgs {
    amount: u64,
    max_sol_cost: u64,
}

// Parse create coin arguments
fn parse_create_coin_args(
    transaction: &VersionedTransaction,
    instruction: &CompiledInstruction,
) -> Result<Transaction, &'static str> {
    if instruction.accounts.len() < 8 {
        return Err("accounts too short");
    }

    let static_keys = transaction.message.static_account_keys();

    let mint_index = instruction.accounts[0] as usize;
    if mint_index >= static_keys.len() {
        return Err("mint index out of bounds");
    }
    let mint = static_keys[mint_index];

    let creator_index = instruction.accounts[7] as usize;
    if creator_index >= static_keys.len() {
        return Err("creator index out of bounds");
    }
    let creator = static_keys[creator_index];

    Ok(Transaction {
        tx_hash: transaction.signatures[0].to_string(),
        maker: creator.to_string(),
        token0_address: mint.to_string(),
        token1_address: WSOL_MINT.to_string(),
        token0_amount: "0".to_string(),
        token1_amount: "0".to_string(),
        program: "pump".to_string(),
        event: "create".to_string(),
    })
}

// Parse buy tokens arguments
fn parse_buy_tokens_args(
    transaction: &VersionedTransaction,
    instruction: &CompiledInstruction,
) -> Result<Transaction, &'static str> {
    let res = if let Ok(args) = BuyArgs::try_from_slice(&instruction.data[8..]) {
        Ok((args.amount, args.max_sol_cost))
    } else if instruction.data.len() >= 24 {
        // try to parse manually
        let amount_bytes = [
            instruction.data[8],
            instruction.data[9],
            instruction.data[10],
            instruction.data[11],
            instruction.data[12],
            instruction.data[13],
            instruction.data[14],
            instruction.data[15],
        ];

        let max_sol_bytes = [
            instruction.data[16],
            instruction.data[17],
            instruction.data[18],
            instruction.data[19],
            instruction.data[20],
            instruction.data[21],
            instruction.data[22],
            instruction.data[23],
        ];

        Ok((
            u64::from_le_bytes(amount_bytes),
            u64::from_le_bytes(max_sol_bytes),
        ))
    } else {
        Err("failed to parse buy tokens args")
    };
    if let Err(e) = res {
        return Err(e);
    }

    let (amount, max_sol) = res.unwrap();

    if instruction.accounts.len() < 8 {
        return Err("accounts too short");
    }

    let static_keys = transaction.message.static_account_keys();

    let mint_index = instruction.accounts[2] as usize;
    if mint_index >= static_keys.len() {
        return Err("mint index out of bounds");
    }
    let mint = static_keys[mint_index];

    let buyer_index = instruction.accounts[7] as usize;
    if buyer_index >= static_keys.len() {
        return Err("buyer index out of bounds");
    }
    let buyer = static_keys[buyer_index];

    Ok(Transaction {
        tx_hash: transaction.signatures[0].to_string(),
        maker: buyer.to_string(),
        token0_address: mint.to_string(),
        token1_address: WSOL_MINT.to_string(),
        token0_amount: format_token_amount(amount),
        token1_amount: format_sol_amount(max_sol),
        program: "pump".to_string(),
        event: "buy".to_string(),
    })
}

// Parse sell tokens arguments
fn parse_sell_tokens_args(
    transaction: &VersionedTransaction,
    instruction: &CompiledInstruction,
) -> Result<Transaction, &'static str> {
    let res = if let Ok(args) = ExtendedSellArgs::try_from_slice(&instruction.data[8..]) {
        Ok((args.amount, args.min_sol_output))
    } else if instruction.data.len() >= 24 {
        // try to parse manually
        let amount_bytes = [
            instruction.data[8],
            instruction.data[9],
            instruction.data[10],
            instruction.data[11],
            instruction.data[12],
            instruction.data[13],
            instruction.data[14],
            instruction.data[15],
        ];

        let min_sol_bytes = [
            instruction.data[16],
            instruction.data[17],
            instruction.data[18],
            instruction.data[19],
            instruction.data[20],
            instruction.data[21],
            instruction.data[22],
            instruction.data[23],
        ];

        Ok((
            u64::from_le_bytes(amount_bytes),
            u64::from_le_bytes(min_sol_bytes),
        ))
    } else {
        Err("failed to parse sell tokens args")
    };

    if let Err(e) = res {
        return Err(e);
    }
    let (amount, min_sol) = res.unwrap();

    if instruction.accounts.len() < 8 {
        return Err("accounts too short");
    }

    let static_keys = transaction.message.static_account_keys();

    let mint_index = instruction.accounts[2] as usize;
    if mint_index >= static_keys.len() {
        return Err("mint index out of bounds");
    }
    let mint = static_keys[mint_index];

    let seller_index = instruction.accounts[7] as usize;
    if seller_index >= static_keys.len() {
        return Err("seller index out of bounds");
    }
    let seller = static_keys[seller_index];

    Ok(Transaction {
        tx_hash: transaction.signatures[0].to_string(),
        maker: seller.to_string(),
        token0_address: mint.to_string(),
        token1_address: WSOL_MINT.to_string(),
        token0_amount: format_token_amount(amount),
        token1_amount: format_sol_amount(min_sol),
        program: "pump".to_string(),
        event: "sell".to_string(),
    })
}

pub fn format_token_amount(amount: u64) -> String {
    let sol = amount as f64 / 1_000_000.0;
    format!("{:.6}", sol)
}

/// Convert lamports to SOL string
pub fn format_sol_amount(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    format!("{:.9}", sol)
}

pub fn parse_pump_instruction(
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
        EXTENDED_SELL_IX => {
            return Some(parse_sell_tokens_args(transaction, instruction));
        }
        CREATE_COIN_IX => {
            return Some(parse_create_coin_args(transaction, instruction));
        }
        _ => None,
    }
}
