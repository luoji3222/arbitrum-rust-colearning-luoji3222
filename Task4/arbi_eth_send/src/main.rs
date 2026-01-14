use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::{env, sync::Arc};
use anyhow::{Context, Result};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载 .env
    dotenv().ok();

    // 1️⃣ 读取配置
    let arb_rpc = env::var("ARB_RPC")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let sender_key = env::var("PRIVKEY").context("🔑 请在 .env 设置 PRIVKEY")?;
    let receiver_addr: Address = env::var("TO_ADDR")
        .context("📬 请在 .env 设置 TO_ADDR")?
        .parse()
        .context("📬 TO_ADDR 格式非法")?;
    let send_value_eth: f64 = env::var("AMOUNT")
        .unwrap_or_else(|_| "0.001".to_string())
        .parse()
        .context("💰 AMOUNT 必须是数字（单位 ETH）")?;
    let manual_gwei = env::var("GAS_PRICE_GWEI")
        .ok()
        .and_then(|s| s.parse::<u64>().ok());

    // 2️⃣ 构建签名器 & 客户端
    let wallet: LocalWallet = sender_key
        .trim_start_matches("0x")
        .parse::<LocalWallet>()?
        .with_chain_id(421614u64);
    let provider = Provider::<Http>::try_from(arb_rpc)?
        .interval(std::time::Duration::from_secs(1));
    let signer_client = Arc::new(SignerMiddleware::new(provider, wallet));

    let sender = signer_client.address();
    println!("\n🚀 Arbitrum Sepolia 转账脚本");
    println!("├─ 发送方: {}", sender);
    println!("├─ 接收方: {}", receiver_addr);
    println!("├─ 金额  : {} ETH", send_value_eth);

    // 3️⃣ 余额检查
    let balance_wei = signer_client.get_balance(sender, None).await?;
    let transfer_wei = ethers::utils::parse_ether(send_value_eth)?;
    if balance_wei < transfer_wei {
        anyhow::bail!(
            "❌ 余额不足：需要 {} ETH，实际 {} ETH",
            send_value_eth,
            ethers::utils::format_ether(balance_wei)
        );
    }

    // 4️⃣ Gas 价格
    let gas_price = if let Some(gwei) = manual_gwei {
        ethers::utils::parse_units(gwei, "gwei")?.into()
    } else {
        let price = signer_client.get_gas_price().await?;
        price * 110_u32 / 100_u32
    };
    println!("├─ GasPrice: {} gwei", ethers::utils::format_units(gas_price, "gwei")?);

    // 5️⃣ 估算 Gas 上限
    let estimate_tx = TransactionRequest::new()
        .to(receiver_addr)
        .value(transfer_wei)
        .from(sender);
    let typed_tx = TypedTransaction::Legacy(estimate_tx);
    let gas_estimate = signer_client.estimate_gas(&typed_tx, None).await?;
    let gas_limit = gas_estimate * 130_u32 / 100_u32;
    println!("├─ GasLimit: {} (估算 {} +30%)", gas_limit, gas_estimate);

    // 6️⃣ 组装最终交易
    let final_tx = TransactionRequest::new()
        .to(receiver_addr)
        .value(transfer_wei)
        .gas_price(gas_price)
        .gas(gas_limit)
        .from(sender);

    // 7️⃣ 签名 & 广播
    let pending = signer_client.send_transaction(final_tx, None).await?;
    let tx_hash = *pending;
    println!("├─ ✍️ 交易已签名，哈希: {:?}", tx_hash);

    // 8️⃣ 等待 1 个确认
    let receipt = pending
        .confirmations(1)
        .await?
        .ok_or_else(|| anyhow::anyhow!("❌ 交易被打回"))?;
    println!("└─ ✅ 上链成功！区块高度: {:?}", receipt.block_number.unwrap());

    Ok(())
}