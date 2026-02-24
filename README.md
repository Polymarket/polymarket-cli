# Polymarket CLI

Rust CLI for Polymarket. Browse markets, place orders, manage positions, and interact with onchain contracts — from a terminal or as a JSON API for scripts and agents.

> **Warning:** This is early, experimental software. Use at your own risk and do not use with large amounts of funds. APIs, commands, and behavior may change without notice. Always verify transactions before confirming.

## Install

```bash
curl -sSL https://raw.githubusercontent.com/Polymarket/polymarket-cli/main/install.sh | sh
```

Or build from source:

```bash
git clone https://github.com/Polymarket/polymarket-cli
cd polymarket-cli
cargo install --path .
```

## Quick Start

```bash
# No wallet needed — browse markets immediately
polymarket markets list --limit 5
polymarket markets search "election"
polymarket events list --tag politics

# Check a specific market
polymarket markets get will-trump-win-the-2024-election

# JSON output for scripts
polymarket -o json markets list --limit 3
```

To trade, set up a wallet:

```bash
polymarket setup
# Or manually:
polymarket wallet create
polymarket approve set
```

## Configuration

### Wallet Setup

The CLI needs a private key to sign orders and on-chain transactions. Three ways to provide it (checked in this order):

1. **CLI flag**: `--private-key 0xabc...`
2. **Environment variable**: `POLYMARKET_PRIVATE_KEY=0xabc...`
3. **Config file**: `~/.config/polymarket/config.json`

```bash
# Create a new wallet (generates random key, saves to config)
polymarket wallet create

# Import an existing key
polymarket wallet import 0xabc123...

# Check what's configured
polymarket wallet show
```

The config file (`~/.config/polymarket/config.json`):

```json
{
  "private_key": "0x...",
  "chain_id": 137,
  "signature_type": "proxy"
}
```

### Signature Types

- `proxy` (default) — uses Polymarket's proxy wallet system
- `eoa` — signs directly with your key
- `gnosis-safe` — for multisig wallets

Override per-command with `--signature-type eoa` or via `POLYMARKET_SIGNATURE_TYPE`.

### What Needs a Wallet

Most commands work without a wallet — browsing markets, viewing order books, checking prices. You only need a wallet for:

- Placing and canceling orders (`clob create-order`, `clob market-order`, `clob cancel-*`)
- Checking your balances and trades (`clob balance`, `clob trades`, `clob orders`)
- On-chain operations (`approve set`, `ctf split/merge/redeem`)
- Reward and API key management (`clob rewards`, `clob create-api-key`)

## Output Formats

Every command supports `--output table` (default) and `--output json`.

```bash
# Human-readable table (default)
polymarket markets list --limit 2
```

```
 Question                            Price        Volume   Liquidity  Status
 Will Trump win the 2024 election?   Yes: 52.00¢  $145.2M  $1.2M      Active
 Will BTC hit $100k by Dec 2024?     Yes: 67.30¢  $89.4M   $430.5K    Active
```

```bash
# Machine-readable JSON
polymarket -o json markets list --limit 2
```

```json
[
  { "id": "12345", "question": "Will Trump win the 2024 election?", "outcomePrices": ["0.52", "0.48"], ... },
  { "id": "67890", "question": "Will BTC hit $100k by Dec 2024?", ... }
]
```

Short form: `-o json` or `-o table`.

Errors follow the same pattern — table mode prints `Error: ...` to stderr, JSON mode prints `{"error": "..."}` to stdout. Non-zero exit code either way.

## Commands

### Markets

```bash
# List markets with filters
polymarket markets list --limit 10
polymarket markets list --active true --order volume_num
polymarket markets list --closed false --limit 50 --offset 25

# Get a single market by ID or slug
polymarket markets get 12345
polymarket markets get will-trump-win

# Search
polymarket markets search "bitcoin" --limit 5

# Get tags for a market
polymarket markets tags 12345
```

**Flags for `markets list`**: `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed`

### Events

Events group related markets (e.g. "2024 Election" contains multiple yes/no markets).

```bash
polymarket events list --limit 10
polymarket events list --tag politics --active true
polymarket events get 500
polymarket events tags 500
```

**Flags for `events list`**: `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed`, `--tag`

### Tags, Series, Comments, Profiles, Sports

```bash
# Tags
polymarket tags list
polymarket tags get politics
polymarket tags related politics
polymarket tags related-tags politics

# Series (recurring events)
polymarket series list --limit 10
polymarket series get 42

# Comments on an entity
polymarket comments list --entity-type event --entity-id 500
polymarket comments get abc123
polymarket comments by-user 0xf5E6...

# Public profiles
polymarket profiles get 0xf5E6...

# Sports metadata
polymarket sports list
polymarket sports market-types
polymarket sports teams --league NFL --limit 32
```

### Order Book & Prices (CLOB)

All read-only — no wallet needed.

```bash
# Check API health
polymarket clob ok

# Prices
polymarket clob price 48331043336612883... --side buy
polymarket clob midpoint 48331043336612883...
polymarket clob spread 48331043336612883...

# Batch queries (comma-separated token IDs)
polymarket clob batch-prices "TOKEN1,TOKEN2" --side buy
polymarket clob midpoints "TOKEN1,TOKEN2"
polymarket clob spreads "TOKEN1,TOKEN2"

# Order book
polymarket clob book 48331043336612883...
polymarket clob books "TOKEN1,TOKEN2"

# Last trade
polymarket clob last-trade 48331043336612883...

# Market info
polymarket clob market 0xABC123...  # by condition ID
polymarket clob markets             # list all

# Price history
polymarket clob price-history 48331043336612883... --interval 1d --fidelity 30

# Metadata
polymarket clob tick-size 48331043336612883...
polymarket clob fee-rate 48331043336612883...
polymarket clob neg-risk 48331043336612883...
polymarket clob time
polymarket clob geoblock
```

**Interval options for `price-history`**: `1m`, `1h`, `6h`, `1d`, `1w`, `max`

### Trading (CLOB, authenticated)

Requires a configured wallet.

```bash
# Place a limit order (buy 10 shares at $0.50)
polymarket clob create-order \
  --token 48331043336612883... \
  --side buy --price 0.50 --size 10

# Place a market order (buy $5 worth)
polymarket clob market-order \
  --token 48331043336612883... \
  --side buy --amount 5

# Post multiple orders at once
polymarket clob post-orders \
  --tokens "TOKEN1,TOKEN2" \
  --side buy \
  --prices "0.40,0.60" \
  --sizes "10,10"

# Cancel
polymarket clob cancel ORDER_ID
polymarket clob cancel-orders "ORDER1,ORDER2"
polymarket clob cancel-market --market 0xCONDITION...
polymarket clob cancel-all

# View your orders and trades
polymarket clob orders
polymarket clob orders --market 0xCONDITION...
polymarket clob order ORDER_ID
polymarket clob trades

# Check balances
polymarket clob balance --asset-type collateral
polymarket clob balance --asset-type conditional --token 48331043336612883...
polymarket clob update-balance --asset-type collateral
```

**Order types**: `GTC` (default), `FOK`, `GTD`, `FAK`. Add `--post-only` for limit orders.

### Rewards & API Keys (CLOB, authenticated)

```bash
polymarket clob rewards --date 2024-06-15
polymarket clob earnings --date 2024-06-15
polymarket clob earnings-markets --date 2024-06-15
polymarket clob reward-percentages
polymarket clob current-rewards
polymarket clob market-reward 0xCONDITION...

# Check if orders are scoring rewards
polymarket clob order-scoring ORDER_ID
polymarket clob orders-scoring "ORDER1,ORDER2"

# API key management
polymarket clob api-keys
polymarket clob create-api-key
polymarket clob delete-api-key

# Account status
polymarket clob account-status
polymarket clob notifications
polymarket clob delete-notifications "NOTIF1,NOTIF2"
```

### On-Chain Data

Public data — no wallet needed.

```bash
# Portfolio
polymarket data positions 0xWALLET_ADDRESS
polymarket data closed-positions 0xWALLET_ADDRESS
polymarket data value 0xWALLET_ADDRESS
polymarket data traded 0xWALLET_ADDRESS

# Trade history
polymarket data trades 0xWALLET_ADDRESS --limit 50

# Activity
polymarket data activity 0xWALLET_ADDRESS

# Market data
polymarket data holders 0xCONDITION_ID
polymarket data open-interest 0xCONDITION_ID
polymarket data volume 12345  # event ID

# Leaderboards
polymarket data leaderboard --period month --order-by pnl --limit 10
polymarket data builder-leaderboard --period week
polymarket data builder-volume --period month
```

### Contract Approvals

Before trading, Polymarket contracts need ERC-20 (USDC) and ERC-1155 (CTF token) approvals.

```bash
# Check current approvals (read-only)
polymarket approve check
polymarket approve check 0xSOME_ADDRESS

# Approve all contracts (sends 6 on-chain transactions, needs MATIC for gas)
polymarket approve set
```

### CTF Operations

Split, merge, and redeem conditional tokens directly on-chain.

```bash
# Split $10 USDC into YES/NO tokens
polymarket ctf split --condition 0xCONDITION... --amount 10

# Merge tokens back to USDC
polymarket ctf merge --condition 0xCONDITION... --amount 10

# Redeem winning tokens after resolution
polymarket ctf redeem --condition 0xCONDITION...

# Redeem neg-risk positions
polymarket ctf redeem-neg-risk --condition 0xCONDITION... --amounts "10,5"

# Calculate IDs (read-only, no wallet needed)
polymarket ctf condition-id --oracle 0xORACLE... --question 0xQUESTION... --outcomes 2
polymarket ctf collection-id --condition 0xCONDITION... --index-set 1
polymarket ctf position-id --collection 0xCOLLECTION...
```

`--amount` is in USDC (e.g., `10` = $10). The `--partition` flag defaults to binary (`1,2`). On-chain operations require MATIC for gas on Polygon.

### Bridge

Deposit assets from other chains into Polymarket.

```bash
# Get deposit addresses (EVM, Solana, Bitcoin)
polymarket bridge deposit 0xWALLET_ADDRESS

# List supported chains and tokens
polymarket bridge supported-assets

# Check deposit status
polymarket bridge status 0xDEPOSIT_ADDRESS
```

### Wallet Management

```bash
polymarket wallet create               # Generate new random wallet
polymarket wallet create --force       # Overwrite existing
polymarket wallet import 0xKEY...      # Import existing key
polymarket wallet address              # Print wallet address
polymarket wallet show                 # Full wallet info (address, source, config path)
polymarket wallet reset                # Delete config (prompts for confirmation)
polymarket wallet reset --force        # Delete without confirmation
```

### Interactive Shell

```bash
polymarket shell
# polymarket> markets list --limit 3
# polymarket> clob book 48331043336612883...
# polymarket> exit
```

Supports command history. All commands work the same as the CLI, just without the `polymarket` prefix.

### Other

```bash
polymarket status     # API health check
polymarket setup      # Guided first-time setup wizard
polymarket upgrade    # Update to the latest version
polymarket --version
polymarket --help
```

## Common Workflows

### Browse and research markets

```bash
polymarket markets search "bitcoin" --limit 5
polymarket markets get bitcoin-above-100k
polymarket clob book 48331043336612883...
polymarket clob price-history 48331043336612883... --interval 1d
```

### Set up a new wallet and start trading

```bash
polymarket wallet create
polymarket approve set                    # needs MATIC for gas
polymarket clob balance --asset-type collateral
polymarket clob market-order --token TOKEN_ID --side buy --amount 5
```

### Monitor your portfolio

```bash
polymarket data positions 0xYOUR_ADDRESS
polymarket data value 0xYOUR_ADDRESS
polymarket clob orders
polymarket clob trades
```

### Place and manage limit orders

```bash
# Place order
polymarket clob create-order --token TOKEN_ID --side buy --price 0.45 --size 20

# Check it
polymarket clob orders

# Cancel if needed
polymarket clob cancel ORDER_ID

# Or cancel everything
polymarket clob cancel-all
```

### Script with JSON output

```bash
# Pipe market data to jq
polymarket -o json markets list --limit 100 | jq '.[].question'

# Check prices programmatically
polymarket -o json clob midpoint TOKEN_ID | jq '.mid'

# Error handling in scripts
if ! result=$(polymarket -o json clob balance --asset-type collateral 2>/dev/null); then
  echo "Failed to fetch balance"
fi
```

## Architecture

```
src/
  main.rs        -- CLI entry point, clap parsing, error handling
  auth.rs        -- Wallet resolution, RPC provider, CLOB authentication
  config.rs      -- Config file (~/.config/polymarket/config.json)
  shell.rs       -- Interactive REPL
  commands/      -- One module per command group
  output/        -- Table and JSON rendering per command group
```

## License

MIT
