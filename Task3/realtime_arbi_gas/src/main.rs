// 导入必要的trait和类型
use ethers::{
    providers::{Http, Provider, Middleware},  // 必须导入Middleware trait
    types::U256,
};
use eyre::Result;
use reqwest::Client;  // 导入reqwest
use url::Url;  // 导入Url类型
use std::time::Duration;

// Arbitrum普通转账的基础Gas限额（行业通用值）
const BASE_TRANSFER_GAS_LIMIT: u64 = 21000;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 构建HTTP客户端（带超时）
    let http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // 2. 解析RPC URL为Url类型（解决From<&str>不满足的问题）
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    let url = Url::parse(rpc_url)?;

    // 3. 初始化Arbitrum Sepolia测试网提供者
    let http_transport = Http::new_with_client(url, http_client);
    let provider = Provider::new(http_transport);

    // 4. 动态获取实时Gas价格（单位：wei）
    println!("正在获取Arbitrum Sepolia实时Gas价格...");
    let gas_price_wei = provider.get_gas_price().await?;
    println!("✅ 实时Gas价格（wei）: {}", gas_price_wei);

    // 转换为gwei（1 gwei = 10^9 wei，更易读）
    let gas_price_gwei = gas_price_wei.as_u64() / 1_000_000_000;
    println!("✅ 实时Gas价格（gwei）: {}", gas_price_gwei);

    // 5. 核心计算：Gas费 = Gas价格 × Gas限额
    let gas_fee_wei = gas_price_wei * U256::from(BASE_TRANSFER_GAS_LIMIT);
    println!("✅ 预估转账Gas费（wei）: {}", gas_fee_wei);

    // 修复：U256没有as_f64()，先转u128再转f64（避免溢出）
    let gas_fee_ether = gas_fee_wei.as_u128() as f64 / 1e18;
    println!("✅ 预估转账Gas费（ether）: {:.8}", gas_fee_ether);

    Ok(())
}