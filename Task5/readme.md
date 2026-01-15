查询链信息<br>
provider.get_chainid()<br>
读取当前 RPC 连接的链 ID<br>
2) 校验地址是不是合约<br>
provider.get_code(token_addr, None)<br>
如果返回 0x（空），说明这个地址在当前链上不是合约（地址填错或链连错）。<br>
3) 对 ERC20 合约做只读调用（view）<br>
通过 contract.method(...).call() 调用了这些函数：<br>
name()<br>
**作用：**读取代币名称（string）<br>
symbol()<br>
**作用：**读取代币符号（string）<br>
decimals()<br>
**作用：**读取小数位（uint8）<br>
totalSupply()<br>
**作用：**读取代币总供应量（uint256）<br>
balanceOf(address)<br>
**作用：**读取某个地址的余额（uint256）<br>
![](./images/1.png)