use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use console::Term;
use dialoguer::{Input, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use lattice_execution::types::Address;
use lattice_wallet::{Wallet, WalletConfig};
use primitive_types::U256;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "lattice-wallet")]
#[command(about = "Lattice blockchain wallet")]
struct Cli {
    /// Wallet keystore path
    #[arg(short, long)]
    keystore: Option<PathBuf>,
    
    /// RPC URL
    #[arg(short, long, default_value = "http://localhost:8545")]
    rpc: String,
    
    /// Chain ID
    #[arg(short, long, default_value = "1337")]
    chain_id: u64,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new account
    New {
        /// Account alias
        #[arg(short, long)]
        alias: Option<String>,
    },
    
    /// Import account from private key
    Import {
        /// Private key in hex format
        #[arg(short, long)]
        key: Option<String>,
        
        /// Account alias
        #[arg(short, long)]
        alias: Option<String>,
    },
    
    /// List all accounts
    List,
    
    /// Show account balance
    Balance {
        /// Account index or address
        account: Option<String>,
    },
    
    /// Send transaction
    Send {
        /// From account index
        #[arg(short, long)]
        from: usize,
        
        /// To address
        #[arg(short, long)]
        to: String,
        
        /// Amount in LATT
        #[arg(short, long)]
        amount: String,
        
        /// Gas price in gwei
        #[arg(short, long)]
        gas_price: Option<u64>,
        
        /// Gas limit
        #[arg(short, long)]
        gas_limit: Option<u64>,
    },
    
    /// Export private key
    Export {
        /// Account index
        index: usize,
    },
    
    /// Show wallet info
    Info,
    
    /// Interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .init();
    
    let cli = Cli::parse();
    
    // Create wallet config
    let mut config = WalletConfig::default();
    if let Some(keystore) = cli.keystore {
        config.keystore_path = keystore;
    }
    config.rpc_url = cli.rpc;
    config.chain_id = cli.chain_id;
    
    // Create wallet
    let mut wallet = Wallet::new(config)?;
    
    match cli.command {
        Commands::New { alias } => {
            create_account(&mut wallet, alias).await?;
        }
        Commands::Import { key, alias } => {
            import_account(&mut wallet, key, alias).await?;
        }
        Commands::List => {
            list_accounts(&mut wallet).await?;
        }
        Commands::Balance { account } => {
            show_balance(&mut wallet, account).await?;
        }
        Commands::Send { from, to, amount, gas_price, gas_limit } => {
            send_transaction(&mut wallet, from, &to, &amount, gas_price, gas_limit).await?;
        }
        Commands::Export { index } => {
            export_key(&mut wallet, index).await?;
        }
        Commands::Info => {
            show_info(&wallet).await?;
        }
        Commands::Interactive => {
            interactive_mode(&mut wallet).await?;
        }
    }
    
    Ok(())
}

async fn create_account(wallet: &mut Wallet, alias: Option<String>) -> Result<()> {
    println!("{}", "Creating new account...".bright_cyan());
    
    let password = Password::new()
        .with_prompt("Enter password for new account")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()?;
    
    let account = wallet.create_account(&password, alias)?;
    
    println!("{}", "✓ Account created successfully!".green());
    println!("  Index:   {}", account.index);
    println!("  Address: 0x{}", hex::encode(&account.address.0));
    println!("  Public:  0x{}", hex::encode(account.public_key.as_bytes()));
    
    if let Some(alias) = &account.alias {
        println!("  Alias:   {}", alias);
    }
    
    Ok(())
}

async fn import_account(wallet: &mut Wallet, key: Option<String>, alias: Option<String>) -> Result<()> {
    println!("{}", "Importing account...".bright_cyan());
    
    let private_key = if let Some(key) = key {
        key
    } else {
        Password::new()
            .with_prompt("Enter private key (hex)")
            .interact()?
    };
    
    let password = Password::new()
        .with_prompt("Enter password to encrypt key")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()?;
    
    let account = wallet.import_account(&private_key, &password, alias)?;
    
    println!("{}", "✓ Account imported successfully!".green());
    println!("  Index:   {}", account.index);
    println!("  Address: 0x{}", hex::encode(&account.address.0));
    
    if let Some(alias) = &account.alias {
        println!("  Alias:   {}", alias);
    }
    
    Ok(())
}

async fn list_accounts(wallet: &mut Wallet) -> Result<()> {
    // Refresh accounts
    wallet.refresh_accounts()?;
    
    if wallet.list_accounts().is_empty() {
        println!("{}", "No accounts found. Create one with 'wallet new'".yellow());
        return Ok(());
    }
    
    // Try to unlock to get balances
    let unlocked = if let Ok(password) = Password::new()
        .with_prompt("Enter password to view balances (or press Enter to skip)")
        .allow_empty_password(true)
        .interact()
    {
        if !password.is_empty() {
            wallet.unlock(&password).is_ok()
        } else {
            false
        }
    } else {
        false
    };
    
    if unlocked {
        // Update balances
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")?);
        pb.set_message("Fetching balances...");
        pb.enable_steady_tick(Duration::from_millis(100));
        
        wallet.update_balances().await?;
        
        pb.finish_and_clear();
    }
    
    let accounts = wallet.list_accounts();
    
    println!("{}", "Accounts:".bright_cyan());
    println!("{}", "─".repeat(80));
    
    for account in accounts {
        println!("  [{}] {}", 
            account.index, 
            account.alias.as_ref().unwrap_or(&"<no alias>".to_string()).bright_yellow()
        );
        println!("      Address: 0x{}", hex::encode(&account.address.0));
        
        if unlocked {
            let balance_latt = format_latt(account.balance);
            println!("      Balance: {} LATT", balance_latt.bright_green());
            println!("      Nonce:   {}", account.nonce);
        }
        
        println!();
    }
    
    Ok(())
}

async fn show_balance(wallet: &mut Wallet, account: Option<String>) -> Result<()> {
    if let Some(acc) = account {
        if let Ok(index) = acc.parse::<usize>() {
            // Show wallet account balance
            wallet.refresh_accounts()?;
            let account = wallet.get_account(index)
                .ok_or_else(|| anyhow::anyhow!("Account not found"))?
                .clone();
            
            // Fetch balance
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")?);
            pb.set_message("Fetching balance...");
            pb.enable_steady_tick(Duration::from_millis(100));
            
            let balance = wallet.rpc_client().get_balance(&account.address).await?;
            let nonce = wallet.rpc_client().get_nonce(&account.address).await?;
            
            pb.finish_and_clear();
            
            println!("{}", "Account Balance:".bright_cyan());
            println!("  Address: 0x{}", hex::encode(&account.address.0));
            
            if let Some(alias) = &account.alias {
                println!("  Alias:   {}", alias);
            }
            
            println!("  Balance: {} LATT", format_latt(balance).bright_green());
            println!("  Nonce:   {}", nonce);
        } else if acc.starts_with("0x") {
            // Show any address balance
            let addr_bytes = hex::decode(&acc[2..])?;
            if addr_bytes.len() != 20 {
                anyhow::bail!("Invalid address length");
            }
            let mut addr_array = [0u8; 20];
            addr_array.copy_from_slice(&addr_bytes);
            let address = Address(addr_array);
            
            // Check if it's a wallet account
            wallet.refresh_accounts()?;
            let account_info = wallet.get_account_by_address(&address);
            
            // Fetch balance
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")?);
            pb.set_message("Fetching balance...");
            pb.enable_steady_tick(Duration::from_millis(100));
            
            let balance = wallet.rpc_client().get_balance(&address).await?;
            let nonce = wallet.rpc_client().get_nonce(&address).await?;
            
            pb.finish_and_clear();
            
            println!("{}", "Address Balance:".bright_cyan());
            println!("  Address: 0x{}", hex::encode(&address.0));
            
            if let Some(account) = account_info {
                if let Some(alias) = &account.alias {
                    println!("  Alias:   {}", alias);
                }
            }
            
            println!("  Balance: {} LATT", format_latt(balance).bright_green());
            println!("  Nonce:   {}", nonce);
        } else {
            anyhow::bail!("Invalid account specifier");
        }
    } else {
        // Show all wallet accounts
        list_accounts(wallet).await?;
    }
    
    Ok(())
}

async fn send_transaction(
    wallet: &mut Wallet,
    from: usize,
    to: &str,
    amount: &str,
    gas_price: Option<u64>,
    gas_limit: Option<u64>,
) -> Result<()> {
    // Parse recipient address
    let to_bytes = hex::decode(to.trim_start_matches("0x"))?;
    if to_bytes.len() != 20 {
        anyhow::bail!("Invalid recipient address");
    }
    let mut to_array = [0u8; 20];
    to_array.copy_from_slice(&to_bytes);
    let to_address = Address(to_array);
    
    // Parse amount
    let amount_latt = amount.parse::<f64>()?;
    let amount_wei = latt_to_wei(amount_latt);
    
    // Unlock wallet
    let password = Password::new()
        .with_prompt("Enter password to unlock wallet")
        .interact()?;
    
    wallet.unlock(&password)?;
    
    // Update balances
    wallet.update_balances().await?;
    
    // Show transaction details
    println!("{}", "Transaction Details:".bright_cyan());
    println!("  From:   Account #{}", from);
    println!("  To:     0x{}", hex::encode(&to_address.0));
    println!("  Amount: {} LATT", amount);
    
    if let Some(gp) = gas_price {
        println!("  Gas Price: {} gwei", gp);
    }
    if let Some(gl) = gas_limit {
        println!("  Gas Limit: {}", gl);
    }
    
    // Confirm
    println!();
    let confirm = dialoguer::Confirm::new()
        .with_prompt("Send transaction?")
        .default(false)
        .interact()?;
    
    if !confirm {
        println!("{}", "Transaction cancelled".yellow());
        return Ok(());
    }
    
    // Send transaction
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);
    pb.set_message("Sending transaction...");
    pb.enable_steady_tick(Duration::from_millis(100));
    
    let tx_hash = wallet.transfer(from, to_address, amount_wei).await?;
    
    pb.finish_and_clear();
    
    println!("{}", "✓ Transaction sent successfully!".green());
    println!("  Hash: 0x{}", hex::encode(tx_hash.as_bytes()));
    
    // Wait for receipt
    pb.set_message("Waiting for confirmation...");
    pb.enable_steady_tick(Duration::from_millis(100));
    
    let mut receipt = None;
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        if let Ok(Some(r)) = wallet.get_transaction_receipt(&tx_hash).await {
            receipt = Some(r);
            break;
        }
    }
    
    pb.finish_and_clear();
    
    if let Some(receipt) = receipt {
        let status = receipt["status"]
            .as_str()
            .map(|s| s == "0x1")
            .unwrap_or(false);
        
        if status {
            println!("{}", "✓ Transaction confirmed!".green());
        } else {
            println!("{}", "✗ Transaction failed!".red());
        }
        
        if let Some(block) = receipt["blockNumber"].as_str() {
            let block_num = u64::from_str_radix(block.trim_start_matches("0x"), 16)?;
            println!("  Block: #{}", block_num);
        }
    } else {
        println!("{}", "⚠ Transaction pending (check later)".yellow());
    }
    
    Ok(())
}

async fn export_key(wallet: &mut Wallet, index: usize) -> Result<()> {
    println!("{}", "⚠ WARNING: Never share your private key!".bright_red());
    
    // Unlock wallet
    let password = Password::new()
        .with_prompt("Enter password to unlock wallet")
        .interact()?;
    
    wallet.unlock(&password)?;
    
    // Export key
    let private_key = wallet.export_private_key(index)?;
    
    println!("{}", "Private key:".bright_cyan());
    println!("  {}", private_key.bright_yellow());
    
    Ok(())
}

async fn show_info(wallet: &Wallet) -> Result<()> {
    let config = wallet.config();
    
    println!("{}", "Wallet Information:".bright_cyan());
    println!("  Keystore: {:?}", config.keystore_path);
    println!("  RPC URL:  {}", config.rpc_url);
    println!("  Chain ID: {}", config.chain_id);
    
    // Try to get chain info
    if let Ok(block_number) = wallet.rpc_client().get_block_number().await {
        println!("  Block:    #{}", block_number);
    }
    
    if let Ok(gas_price) = wallet.rpc_client().get_gas_price().await {
        println!("  Gas:      {} gwei", gas_price / 1_000_000_000);
    }
    
    Ok(())
}

async fn interactive_mode(wallet: &mut Wallet) -> Result<()> {
    let term = Term::stdout();
    
    loop {
        term.clear_screen()?;
        
        println!("{}", "Lattice Wallet - Interactive Mode".bright_cyan().bold());
        println!("{}", "─".repeat(50));
        
        let options = vec![
            "Create new account",
            "Import account",
            "List accounts",
            "Check balance",
            "Send transaction",
            "Export private key",
            "Wallet info",
            "Exit",
        ];
        
        let selection = Select::new()
            .with_prompt("Select an option")
            .items(&options)
            .default(0)
            .interact()?;
        
        match selection {
            0 => {
                let alias = Input::<String>::new()
                    .with_prompt("Account alias (optional)")
                    .allow_empty(true)
                    .interact()?;
                
                let alias = if alias.is_empty() { None } else { Some(alias) };
                create_account(wallet, alias).await?;
            }
            1 => {
                import_account(wallet, None, None).await?;
            }
            2 => {
                list_accounts(wallet).await?;
            }
            3 => {
                let account = Input::<String>::new()
                    .with_prompt("Account index or address (or Enter for all)")
                    .allow_empty(true)
                    .interact()?;
                
                let account = if account.is_empty() { None } else { Some(account) };
                show_balance(wallet, account).await?;
            }
            4 => {
                let from = Input::<usize>::new()
                    .with_prompt("From account index")
                    .interact()?;
                
                let to = Input::<String>::new()
                    .with_prompt("To address (0x...)")
                    .interact()?;
                
                let amount = Input::<String>::new()
                    .with_prompt("Amount in LATT")
                    .interact()?;
                
                send_transaction(wallet, from, &to, &amount, None, None).await?;
            }
            5 => {
                let index = Input::<usize>::new()
                    .with_prompt("Account index")
                    .interact()?;
                
                export_key(wallet, index).await?;
            }
            6 => {
                show_info(wallet).await?;
            }
            7 => {
                println!("{}", "Goodbye!".bright_green());
                break;
            }
            _ => {}
        }
        
        if selection != 7 {
            println!();
            println!("Press Enter to continue...");
            term.read_line()?;
        }
    }
    
    Ok(())
}

/// Format U256 wei to LATT string
fn format_latt(wei: U256) -> String {
    let decimals = U256::from(10).pow(U256::from(18));
    let whole = wei / decimals;
    let fraction = wei % decimals;
    
    // Format with up to 6 decimal places
    let fraction_str = format!("{:018}", fraction);
    let fraction_trimmed = if fraction_str.len() >= 6 {
        fraction_str[..6].trim_end_matches('0')
    } else {
        fraction_str.trim_end_matches('0')
    };
    
    if fraction_trimmed.is_empty() {
        format!("{}", whole)
    } else {
        format!("{}.{}", whole, fraction_trimmed)
    }
}

/// Convert LATT to wei
fn latt_to_wei(latt: f64) -> U256 {
    let wei_per_latt = 1_000_000_000_000_000_000u128; // 10^18
    let wei = (latt * wei_per_latt as f64) as u128;
    U256::from(wei)
}