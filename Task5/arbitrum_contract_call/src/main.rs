use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Middleware, Provider},
    types::{Address, U256},
};
use std::sync::Arc;

// 测试用格式化：U256 -> f64（注意：超大数可能会溢出/精度丢失，仅用于演示）
fn format_token_amount(amount: U256, decimals: u8) -> f64 {
    let amount_u128 = amount.as_u128();
    amount_u128 as f64 / 10f64.powi(decimals as i32)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Arbitrum Sepolia RPC
    const ARBITRUM_SEPOLIA_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";

    // 2) 目标合约地址：Arbitrum Sepolia 的 Wrapped Native Token (WETH)
    // 来源：Uniswap docs 的 Arbitrum Sepolia wrapped native token 地址
    const TOKEN_ADDRESS: &str = "0x980B62Da83eFf3D4576C647993b0c1D7faf17c73";

    // 3) 最小 ERC20 ABI（包含你要调用的方法）
    const ERC20_ABI_JSON: &str = r#"
        [
            {
                "inputs": [],
                "name": "name",
                "outputs": [{"internalType": "string", "name": "", "type": "string"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "symbol",
                "outputs": [{"internalType": "string", "name": "", "type": "string"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "decimals",
                "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "totalSupply",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
                "name": "balanceOf",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]
    "#;

    // 4) 连接 Provider
    let provider = Provider::<Http>::try_from(ARBITRUM_SEPOLIA_RPC)?;
    let provider = Arc::new(provider);

    // 5) 打印 chainId（Arbitrum Sepolia 应该是 421614）
    let chain_id = provider.get_chainid().await?;
    println!("✅ connected. chain_id = {}", chain_id);

    // 6) 合约地址 + 校验合约代码存在
    let token_addr: Address = TOKEN_ADDRESS.parse()?;
    let code = provider.get_code(token_addr, None).await?;
    if code.is_empty() {
        return Err(format!(
            "❌ 地址 {} 在当前链上没有合约代码（get_code=0x）。\n\
             请确认：\n\
             1) RPC 是否连到 Arbitrum Sepolia (chainId=421614)\n\
             2) TOKEN_ADDRESS 是否确实部署在 Arbitrum Sepolia 上",
            TOKEN_ADDRESS
        )
            .into());
    }

    // 7) 解析 ABI + 创建合约对象
    let abi: Abi = serde_json::from_str(ERC20_ABI_JSON)?;
    let contract = Contract::new(token_addr, abi, provider.clone());

    // 8) 调用只读方法：name / symbol / decimals
    let name: String = contract.method("name", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    let decimals: u8 = contract.method("decimals", ())?.call().await?;

    println!("token address: {:?}", token_addr);
    println!("name    : {}", name);
    println!("symbol  : {}", symbol);
    println!("decimals: {}", decimals);

    // 9) totalSupply
    let total_supply: U256 = contract.method("totalSupply", ())?.call().await?;
    println!("totalSupply(raw)      : {}", total_supply);
    println!(
        "totalSupply(formatted): {} {}",
        format_token_amount(total_supply, decimals),
        symbol
    );

    // 10) balanceOf（随便查一个地址的余额）
    let test_address: Address = "0x66BD2C13FC975E20f95aC3e7fAC9fBe4401F2317".parse()?;
    let balance: U256 = contract.method("balanceOf", test_address)?.call().await?;
    println!("balanceOf(raw)        : {}", balance);
    println!(
        "balanceOf(formatted)  : {} {}",
        format_token_amount(balance, decimals),
        symbol
    );

    Ok(())
}
