# polymarket-cli command reference

## Global flags
- `-h, --help` — show help
- `--version` — print version
- `-o, --output <table|json>` — output format (default: table)
- `--private-key <key>` — private key for signing
- `--signature-type <proxy|eoa|gnosis-safe>` — override signature type

## Environment variables
- `POLYMARKET_PRIVATE_KEY` — private key (checked if no flag provided)
- `POLYMARKET_SIGNATURE_TYPE` — override signature type

## Commands

### markets
- `markets list [--limit N] [--offset N] [--order <field>] [--ascending] [--active true|false] [--closed true|false]` — list markets with filters
- `markets get <id|slug>` — get a single market by ID or slug
- `markets search "<query>" [--limit N]` — search markets by keyword
- `markets tags <market_id>` — get tags for a market

### events
- `events list [--limit N] [--offset N] [--order <field>] [--ascending] [--active true|false] [--closed true|false] [--tag <tag>]` — list events
- `events get <event_id>` — get a single event
- `events tags <event_id>` — get tags for an event

### tags
- `tags list` — list all tags
- `tags get <tag>` — get tag details
- `tags related <tag>` — related markets
- `tags related-tags <tag>` — related tags

### series
- `series list [--limit N]` — list recurring event series
- `series get <series_id>` — get a single series

### comments
- `comments list --entity-type <event|market> --entity-id <id>` — list comments
- `comments get <comment_id>` — get a single comment
- `comments by-user <address>` — comments by a user

### profiles
- `profiles get <address>` — get a public profile

### sports
- `sports list` — list sports
- `sports market-types` — list market types
- `sports teams --league <league> [--limit N]` — list teams by league

### clob (read-only, no wallet needed)
- `clob ok` — API health check
- `clob price <token_id> --side <buy|sell>` — get price
- `clob midpoint <token_id>` — get midpoint price
- `clob spread <token_id>` — get spread
- `clob batch-prices "<token1>,<token2>" --side <buy|sell>` — batch prices
- `clob midpoints "<token1>,<token2>"` — batch midpoints
- `clob spreads "<token1>,<token2>"` — batch spreads
- `clob book <token_id>` — order book for a token
- `clob books "<token1>,<token2>"` — batch order books
- `clob last-trade <token_id>` — last trade price
- `clob market <condition_id>` — market info by condition ID
- `clob markets` — list all CLOB markets
- `clob price-history <token_id> --interval <1m|1h|6h|1d|1w|max> [--fidelity N]` — price history
- `clob tick-size <token_id>` — tick size
- `clob fee-rate <token_id>` — fee rate
- `clob neg-risk <token_id>` — neg-risk status
- `clob time` — server time
- `clob geoblock` — geoblock status

### clob (authenticated, wallet required)
- `clob create-order --token <id> --side <buy|sell> --price <0-1> --size <N> [--type GTC|FOK|GTD|FAK] [--post-only]` — place a limit order
- `clob market-order --token <id> --side <buy|sell> --amount <N>` — place a market order
- `clob post-orders --tokens "<t1>,<t2>" --side <buy|sell> --prices "<p1>,<p2>" --sizes "<s1>,<s2>"` — batch orders
- `clob cancel <order_id>` — cancel a single order
- `clob cancel-orders "<id1>,<id2>"` — cancel multiple orders
- `clob cancel-market --market <condition_id>` — cancel all orders for a market
- `clob cancel-all` — cancel all open orders
- `clob orders [--market <condition_id>]` — list your orders
- `clob order <order_id>` — get a single order
- `clob trades` — list your trades
- `clob balance --asset-type <collateral|conditional> [--token <id>]` — check balance
- `clob update-balance --asset-type <collateral>` — refresh balance

### clob rewards & API keys (authenticated)
- `clob rewards --date <YYYY-MM-DD>` — rewards for a date
- `clob earnings --date <YYYY-MM-DD>` — earnings for a date
- `clob earnings-markets --date <YYYY-MM-DD>` — earnings by market
- `clob reward-percentages` — reward percentages
- `clob current-rewards` — current reward rates
- `clob market-reward <condition_id>` — reward for a market
- `clob order-scoring <order_id>` — check if order is scoring rewards
- `clob orders-scoring "<id1>,<id2>"` — batch scoring check
- `clob api-keys` — list API keys
- `clob create-api-key` — create an API key
- `clob delete-api-key` — delete an API key
- `clob account-status` — account status
- `clob notifications` — list notifications
- `clob delete-notifications "<id1>,<id2>"` — delete notifications

### data (public, no wallet needed)
- `data positions <address>` — open positions
- `data closed-positions <address>` — closed positions
- `data value <address>` — portfolio value
- `data traded <address>` — traded status
- `data trades <address> [--limit N]` — trade history
- `data activity <address>` — activity feed
- `data holders <condition_id>` — token holders
- `data open-interest <condition_id>` — open interest
- `data volume <event_id>` — event volume
- `data leaderboard --period <day|week|month|all> --order-by <pnl|volume> [--limit N]` — leaderboard
- `data builder-leaderboard --period <day|week|month|all>` — builder leaderboard
- `data builder-volume --period <day|week|month|all>` — builder volume

### approve
- `approve check [<address>]` — check current approvals (read-only)
- `approve set` — approve all Polymarket contracts (6 on-chain txns, needs MATIC)

### ctf (on-chain, wallet required)
- `ctf split --condition <id> --amount <N> [--partition <indices>]` — split USDC into YES/NO tokens
- `ctf merge --condition <id> --amount <N> [--partition <indices>]` — merge tokens back to USDC
- `ctf redeem --condition <id>` — redeem winning tokens
- `ctf redeem-neg-risk --condition <id> --amounts "<a1>,<a2>"` — redeem neg-risk positions
- `ctf condition-id --oracle <addr> --question <id> --outcomes <N>` — calculate condition ID (read-only)
- `ctf collection-id --condition <id> --index-set <N>` — calculate collection ID (read-only)
- `ctf position-id --collection <id>` — calculate position ID (read-only)

### bridge
- `bridge deposit <address>` — get deposit addresses (EVM, Solana, Bitcoin)
- `bridge supported-assets` — list supported chains and tokens
- `bridge status <deposit_address>` — check deposit status

### wallet
- `wallet create [--force]` — generate new random wallet
- `wallet import <key>` — import existing private key
- `wallet address` — print wallet address
- `wallet show` — full wallet info (address, source, config path)
- `wallet reset [--force]` — delete config (prompts confirmation)

### other
- `setup` — guided first-time setup wizard
- `shell` — interactive REPL (all commands without `polymarket` prefix)
- `status` — API health check
- `upgrade` — update to latest version

## Config
- Config file: `~/.config/polymarket/config.json`
- Fields: `private_key` (hex with 0x), `chain_id` (137), `signature_type` (proxy|eoa|gnosis-safe)
- Key precedence: `--private-key` flag > `POLYMARKET_PRIVATE_KEY` env > config file

## Notes
- Amounts in `ctf` commands are in USDC (e.g., `10` = $10)
- Default partition for split/merge is binary (`1,2`)
- On-chain operations require MATIC for gas on Polygon (chain ID 137)
- Price values for orders are between 0 and 1 (e.g., 0.50 = 50 cents)
- Token IDs are long numeric strings (e.g., `48331043336612883...`)
- Condition IDs are hex strings (e.g., `0xABC123...`)
