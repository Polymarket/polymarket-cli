[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md)

# Polymarket CLI

Polymarket 的 Rust 命令行工具。浏览市场、下单、管理持仓、与链上合约交互 —— 既可以在终端中使用，也可以作为脚本和代理的 JSON API。

> **警告：** 这是一个早期实验性软件。请自行承担风险，不要用于大额资金。API、命令和行为可能会随时更改。在确认前请务必验证交易。

## 安装

### Homebrew (macOS / Linux)

```bash
brew tap Polymarket/polymarket-cli https://github.com/Polymarket/polymarket-cli
brew install polymarket
```

### Shell 脚本

```bash
curl -sSL https://raw.githubusercontent.com/Polymarket/polymarket-cli/main/install.sh | sh
```

### 从源码构建

```bash
git clone https://github.com/Polymarket/polymarket-cli
cd polymarket-cli
cargo install --path .
```

## 快速开始

```bash
# 无需钱包 —— 立即浏览市场
polymarket markets list --limit 5
polymarket markets search "election"
polymarket events list --tag politics

# 查看特定市场
polymarket markets get will-trump-win-the-2024-election

# 用于脚本的 JSON 输出
polymarket -o json markets list --limit 3
```

要进行交易，请设置钱包：

```bash
polymarket setup
# 或手动设置：
polymarket wallet create
polymarket approve set
```

## 配置

### 钱包设置

CLI 需要私钥来签署订单和链上交易。提供私钥的三种方式（按检查顺序）：

1. **CLI 参数**: `--private-key 0xabc...`
2. **环境变量**: `POLYMARKET_PRIVATE_KEY=0xabc...`
3. **配置文件**: `~/.config/polymarket/config.json`

```bash
# 创建新钱包（生成随机密钥，保存到配置文件）
polymarket wallet create

# 导入现有密钥
polymarket wallet import 0xabc123...

# 查看当前配置
polymarket wallet show
```

配置文件 (`~/.config/polymarket/config.json`)：

```json
{
  "private_key": "0x...",
  "chain_id": 137,
  "signature_type": "proxy"
}
```

### 签名类型

- `proxy`（默认）—— 使用 Polymarket 的代理钱包系统
- `eoa` —— 直接使用您的密钥签名
- `gnosis-safe` —— 用于多签钱包

可通过 `--signature-type eoa` 或 `POLYMARKET_SIGNATURE_TYPE` 环境变量覆盖。

### 哪些操作需要钱包

大多数命令无需钱包即可使用 —— 浏览市场、查看订单簿、查询价格。只有以下操作需要钱包：

- 下单和取消订单 (`clob create-order`, `clob market-order`, `clob cancel-*`)
- 查询余额和交易记录 (`clob balance`, `clob trades`, `clob orders`)
- 链上操作 (`approve set`, `ctf split/merge/redeem`)
- 奖励和 API 密钥管理 (`clob rewards`, `clob create-api-key`)

## 输出格式

每个命令都支持 `--output table`（默认）和 `--output json`。

```bash
# 人类可读的表格格式（默认）
polymarket markets list --limit 2
```

```
 Question                            Price (Yes)  Volume   Liquidity  Status
 Will Trump win the 2024 election?   52.00¢       $145.2M  $1.2M      Active
 Will BTC hit $100k by Dec 2024?     67.30¢       $89.4M   $430.5K    Active
```

```bash
# 机器可读的 JSON 格式
polymarket -o json markets list --limit 2
```

```json
[
  { "id": "12345", "question": "Will Trump win the 2024 election?", "outcomePrices": ["0.52", "0.48"], ... },
  { "id": "67890", "question": "Will BTC hit $100k by Dec 2024?", ... }
]
```

简写形式：`-o json` 或 `-o table`。

错误输出遵循相同模式 —— 表格模式下向 stderr 打印 `Error: ...`，JSON 模式下向 stdout 打印 `{"error": "..."}`。两种情况都返回非零退出码。

## 命令

### 市场

```bash
# 列出市场（带过滤）
polymarket markets list --limit 10
polymarket markets list --active true --order volume_num
polymarket markets list --closed false --limit 50 --offset 25

# 通过 ID 或 slug 获取单个市场
polymarket markets get 12345
polymarket markets get will-trump-win

# 搜索
polymarket markets search "bitcoin" --limit 5

# 获取市场的标签
polymarket markets tags 12345
```

**`markets list` 参数**: `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed`

### 事件

事件将相关市场分组（例如 "2024 选举" 包含多个是/否市场）。

```bash
polymarket events list --limit 10
polymarket events list --tag politics --active true
polymarket events get 500
polymarket events tags 500
```

**`events list` 参数**: `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed`, `--tag`

### 标签、系列、评论、个人资料、体育

```bash
# 标签
polymarket tags list
polymarket tags get politics
polymarket tags related politics
polymarket tags related-tags politics

# 系列（周期性事件）
polymarket series list --limit 10
polymarket series get 42

# 实体的评论
polymarket comments list --entity-type event --entity-id 500
polymarket comments get abc123
polymarket comments by-user 0xf5E6...

# 公开个人资料
polymarket profiles get 0xf5E6...

# 体育元数据
polymarket sports list
polymarket sports market-types
polymarket sports teams --league NFL --limit 32
```

### 订单簿与价格 (CLOB)

所有命令都是只读的 —— 无需钱包。

```bash
# 检查 API 健康状态
polymarket clob ok

# 价格
polymarket clob price 48331043336612883... --side buy
polymarket clob midpoint 48331043336612883...
polymarket clob spread 48331043336612883...

# 批量查询（逗号分隔的代币 ID）
polymarket clob batch-prices "TOKEN1,TOKEN2" --side buy
polymarket clob midpoints "TOKEN1,TOKEN2"
polymarket clob spreads "TOKEN1,TOKEN2"

# 订单簿
polymarket clob book 48331043336612883...
polymarket clob books "TOKEN1,TOKEN2"

# 最近交易
polymarket clob last-trade 48331043336612883...

# 市场信息
polymarket clob market 0xABC123...  # 按 condition ID
polymarket clob markets             # 列出所有

# 价格历史
polymarket clob price-history 48331043336612883... --interval 1d --fidelity 30

# 元数据
polymarket clob tick-size 48331043336612883...
polymarket clob fee-rate 48331043336612883...
polymarket clob neg-risk 48331043336612883...
polymarket clob time
polymarket clob geoblock
```

**`price-history` 时间间隔选项**: `1m`, `1h`, `6h`, `1d`, `1w`, `max`

### 交易 (CLOB，需认证)

需要配置钱包。

```bash
# 下限价单（以 $0.50 买入 10 份）
polymarket clob create-order \
  --token 48331043336612883... \
  --side buy --price 0.50 --size 10

# 下市价单（买入 $5 价值）
polymarket clob market-order \
  --token 48331043336612883... \
  --side buy --amount 5

# 同时发布多个订单
polymarket clob post-orders \
  --tokens "TOKEN1,TOKEN2" \
  --side buy \
  --prices "0.40,0.60" \
  --sizes "10,10"

# 取消订单
polymarket clob cancel ORDER_ID
polymarket clob cancel-orders "ORDER1,ORDER2"
polymarket clob cancel-market --market 0xCONDITION...
polymarket clob cancel-all

# 查看订单和交易记录
polymarket clob orders
polymarket clob orders --market 0xCONDITION...
polymarket clob order ORDER_ID
polymarket clob trades

# 查询余额
polymarket clob balance --asset-type collateral
polymarket clob balance --asset-type conditional --token 48331043336612883...
polymarket clob update-balance --asset-type collateral
```

**订单类型**: `GTC`（默认）、`FOK`、`GTD`、`FAK`。限价单可添加 `--post-only`。

### 奖励与 API 密钥 (CLOB，需认证)

```bash
polymarket clob rewards --date 2024-06-15
polymarket clob earnings --date 2024-06-15
polymarket clob earnings-markets --date 2024-06-15
polymarket clob reward-percentages
polymarket clob current-rewards
polymarket clob market-reward 0xCONDITION...

# 检查订单是否获得奖励
polymarket clob order-scoring ORDER_ID
polymarket clob orders-scoring "ORDER1,ORDER2"

# API 密钥管理
polymarket clob api-keys
polymarket clob create-api-key
polymarket clob delete-api-key

# 账户状态
polymarket clob account-status
polymarket clob notifications
polymarket clob delete-notifications "NOTIF1,NOTIF2"
```

### 链上数据

公开数据 —— 无需钱包。

```bash
# 投资组合
polymarket data positions 0xWALLET_ADDRESS
polymarket data closed-positions 0xWALLET_ADDRESS
polymarket data value 0xWALLET_ADDRESS
polymarket data traded 0xWALLET_ADDRESS

# 交易历史
polymarket data trades 0xWALLET_ADDRESS --limit 50

# 活动记录
polymarket data activity 0xWALLET_ADDRESS

# 市场数据
polymarket data holders 0xCONDITION_ID
polymarket data open-interest 0xCONDITION_ID
polymarket data volume 12345  # event ID

# 排行榜
polymarket data leaderboard --period month --order-by pnl --limit 10
polymarket data builder-leaderboard --period week
polymarket data builder-volume --period month
```

### 合约授权

交易前，Polymarket 合约需要 ERC-20 (USDC) 和 ERC-1155 (CTF 代币) 授权。

```bash
# 检查当前授权状态（只读）
polymarket approve check
polymarket approve check 0xSOME_ADDRESS

# 授权所有合约（发送 6 笔链上交易，需要 MATIC 支付 gas）
polymarket approve set
```

### CTF 操作

直接在链上拆分、合并和赎回条件代币。

```bash
# 将 $10 USDC 拆分为 YES/NO 代币
polymarket ctf split --condition 0xCONDITION... --amount 10

# 将代币合并回 USDC
polymarket ctf merge --condition 0xCONDITION... --amount 10

# 结算后赎回获胜代币
polymarket ctf redeem --condition 0xCONDITION...

# 赎回负风险持仓
polymarket ctf redeem-neg-risk --condition 0xCONDITION... --amounts "10,5"

# 计算 ID（只读，无需钱包）
polymarket ctf condition-id --oracle 0xORACLE... --question 0xQUESTION... --outcomes 2
polymarket ctf collection-id --condition 0xCONDITION... --index-set 1
polymarket ctf position-id --collection 0xCOLLECTION...
```

`--amount` 以 USDC 为单位（例如 `10` = $10）。`--partition` 参数默认为二进制 (`1,2`)。链上操作需要在 Polygon 上支付 MATIC 作为 gas。

### 跨链桥

从其他链向 Polymarket 存入资产。

```bash
# 获取存款地址（EVM、Solana、Bitcoin）
polymarket bridge deposit 0xWALLET_ADDRESS

# 列出支持的链和代币
polymarket bridge supported-assets

# 检查存款状态
polymarket bridge status 0xDEPOSIT_ADDRESS
```

### 钱包管理

```bash
polymarket wallet create               # 生成新的随机钱包
polymarket wallet create --force       # 覆盖现有钱包
polymarket wallet import 0xKEY...      # 导入现有密钥
polymarket wallet address              # 打印钱包地址
polymarket wallet show                 # 完整钱包信息（地址、来源、配置路径）
polymarket wallet reset                # 删除配置（会提示确认）
polymarket wallet reset --force        # 无需确认直接删除
```

### 交互式 Shell

```bash
polymarket shell
# polymarket> markets list --limit 3
# polymarket> clob book 48331043336612883...
# polymarket> exit
```

支持命令历史记录。所有命令与 CLI 相同，只是不需要 `polymarket` 前缀。

### 其他

```bash
polymarket status     # API 健康检查
polymarket setup      # 引导式首次设置向导
polymarket upgrade    # 更新到最新版本
polymarket --version
polymarket --help
```

## 常用工作流程

### 浏览和研究市场

```bash
polymarket markets search "bitcoin" --limit 5
polymarket markets get bitcoin-above-100k
polymarket clob book 48331043336612883...
polymarket clob price-history 48331043336612883... --interval 1d
```

### 设置新钱包并开始交易

```bash
polymarket wallet create
polymarket approve set                    # 需要 MATIC 支付 gas
polymarket clob balance --asset-type collateral
polymarket clob market-order --token TOKEN_ID --side buy --amount 5
```

### 监控投资组合

```bash
polymarket data positions 0xYOUR_ADDRESS
polymarket data value 0xYOUR_ADDRESS
polymarket clob orders
polymarket clob trades
```

### 下单和管理限价单

```bash
# 下单
polymarket clob create-order --token TOKEN_ID --side buy --price 0.45 --size 20

# 查看
polymarket clob orders

# 需要时取消
polymarket clob cancel ORDER_ID

# 或取消所有订单
polymarket clob cancel-all
```

### 使用 JSON 输出编写脚本

```bash
# 通过管道将市场数据传给 jq
polymarket -o json markets list --limit 100 | jq '.[].question'

# 编程方式查询价格
polymarket -o json clob midpoint TOKEN_ID | jq '.mid'

# 脚本中的错误处理
if ! result=$(polymarket -o json clob balance --asset-type collateral 2>/dev/null); then
  echo "获取余额失败"
fi
```

## 架构

```
src/
  main.rs        -- CLI 入口点，clap 解析，错误处理
  auth.rs        -- 钱包解析，RPC 提供者，CLOB 认证
  config.rs      -- 配置文件 (~/.config/polymarket/config.json)
  shell.rs       -- 交互式 REPL
  commands/      -- 每个命令组一个模块
  output/        -- 每个命令组的表格和 JSON 渲染
```

## 许可证

MIT
