---
name: polymarket-cli
description: Browse prediction markets, place orders, manage positions, and interact with Polymarket onchain contracts via the polymarket CLI. Use when a user asks to search markets, check prices, trade, manage wallets, view portfolios, or translate a task into safe polymarket-cli commands with correct flags, output format, and confirmations.
---

# Polymarket CLI

## Overview

polymarket-cli is a Rust terminal tool for interacting with [Polymarket](https://polymarket.com). It supports browsing markets, placing limit and market orders, managing wallets and positions, interacting with on-chain conditional token contracts, and retrieving leaderboard and portfolio data. Every command supports table (default) and JSON output.

> **Warning:** This is early, experimental software. Do not use with large amounts of funds. APIs, commands, and behavior may change without notice. Always verify transactions before confirming.

## Installation

```bash
# Install script (macOS/Linux, verifies SHA256 checksum)
curl -sSL https://raw.githubusercontent.com/Polymarket/polymarket-cli/main/install.sh | sh

# Build from source
git clone https://github.com/Polymarket/polymarket-cli
cd polymarket-cli && cargo install --path .

# Upgrade
polymarket upgrade
```

## Defaults and safety

- Confirm the target wallet and resolve key precedence: `--private-key` flag > `POLYMARKET_PRIVATE_KEY` env var > config file (`~/.config/polymarket/config.json`).
- Never pass private keys in flags when possible; prefer the config file or env var.
- Require confirmation for destructive or financial actions (placing orders, canceling orders, approving contracts, splitting/merging/redeeming tokens).
- Trading commands require USDC balance on Polygon. On-chain operations (`approve set`, `ctf split/merge/redeem`) require MATIC for gas.
- Default signature type is `proxy`. Override with `--signature-type eoa` or `--signature-type gnosis-safe` only when the user explicitly requests it.
- Choose output format: table by default, `-o json` for structured output.

## What needs a wallet

Most commands work without a wallet (browsing markets, viewing order books, checking prices, on-chain data lookups). A wallet is required only for:
- Placing and canceling orders (`clob create-order`, `clob market-order`, `clob cancel-*`)
- Checking your balances and trades (`clob balance`, `clob trades`, `clob orders`)
- On-chain operations (`approve set`, `ctf split/merge/redeem`)
- Reward and API key management (`clob rewards`, `clob create-api-key`)

## Quick start

```bash
# Browse markets (no wallet needed)
polymarket markets list --limit 5
polymarket markets search "election"
polymarket events list --tag politics

# Check a specific market
polymarket markets get will-trump-win-the-2024-election

# JSON output for scripts
polymarket -o json markets list --limit 3

# Set up a wallet to trade
polymarket setup
# Or manually:
polymarket wallet create
polymarket approve set
```

## Task guidance

### Browsing and research
- Use `markets list` with `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed` for filtered browsing.
- Use `markets search "<query>"` to find markets by keyword.
- Use `markets get <id-or-slug>` for a single market detail.
- Use `events list` with `--tag` to browse grouped markets by topic.
- Use `clob book <token_id>` and `clob price-history <token_id> --interval 1d` for order book and price data.

### Trading
- Use `clob create-order --token <id> --side buy --price 0.50 --size 10` for limit orders.
- Use `clob market-order --token <id> --side buy --amount 5` for market orders.
- Use `clob post-orders` with `--tokens`, `--prices`, `--sizes` for batch orders.
- Order types: `GTC` (default), `FOK`, `GTD`, `FAK`. Add `--post-only` for limit orders.
- Cancel: `clob cancel <order_id>`, `clob cancel-orders "<id1>,<id2>"`, `clob cancel-market --market <condition_id>`, `clob cancel-all`.

### Portfolio monitoring
- Use `data positions <address>` and `data value <address>` for portfolio info.
- Use `clob orders` and `clob trades` to view your order and trade history.
- Use `clob balance --asset-type collateral` to check USDC balance.
- Use `clob balance --asset-type conditional --token <id>` to check token balances.

### Wallet management
- Use `wallet create` to generate a new random wallet (saved to config).
- Use `wallet import <key>` to import an existing key.
- Use `wallet show` to display current wallet info.
- Use `wallet reset` to delete config (prompts confirmation; `--force` to skip).

### On-chain operations
- Use `approve check` to view current contract approvals (read-only).
- Use `approve set` to approve all contracts (sends 6 on-chain txns, needs MATIC).
- Use `ctf split --condition <id> --amount 10` to split USDC into YES/NO tokens.
- Use `ctf merge --condition <id> --amount 10` to merge tokens back to USDC.
- Use `ctf redeem --condition <id>` to redeem winning tokens after resolution.
- `--amount` is in USDC (e.g., `10` = $10). On-chain operations require MATIC for gas on Polygon.

### Scripting with JSON output
- Use `-o json` for machine-readable output: `polymarket -o json markets list --limit 100 | jq '.[].question'`.
- Errors in JSON mode print `{"error": "..."}` to stdout with non-zero exit code.
- Table mode prints `Error: ...` to stderr.

## Configuration

Config file: `~/.config/polymarket/config.json`

```json
{
  "private_key": "0x...",
  "chain_id": 137,
  "signature_type": "proxy"
}
```

| Field | Description |
|-------|-------------|
| private_key | Ethereum private key (hex with 0x prefix) |
| chain_id | 137 (Polygon mainnet) |
| signature_type | `proxy` (default), `eoa`, or `gnosis-safe` |

Override signature type per-command with `--signature-type` or via `POLYMARKET_SIGNATURE_TYPE` env var.

## Reference

Read `references/commands.md` for the full command and flag reference.
