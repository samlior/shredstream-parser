pub const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

pub fn format_token_amount(amount: u64) -> String {
    let sol = amount as f64 / 1_000_000.0;
    format!("{:.6}", sol)
}

pub fn format_sol_amount(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    format!("{:.9}", sol)
}
