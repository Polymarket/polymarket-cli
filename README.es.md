[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md)

# Polymarket CLI

CLI de Rust para Polymarket. Explora mercados, realiza órdenes, gestiona posiciones e interactúa con contratos on-chain — desde una terminal o como API JSON para scripts y agentes.

> **Advertencia:** Este es software experimental en etapa temprana. Úsalo bajo tu propia responsabilidad y no lo uses con grandes cantidades de fondos. Las APIs, comandos y comportamientos pueden cambiar sin previo aviso. Siempre verifica las transacciones antes de confirmar.

## Instalación

### Homebrew (macOS / Linux)

```bash
brew tap Polymarket/polymarket-cli https://github.com/Polymarket/polymarket-cli
brew install polymarket
```

### Script de Shell

```bash
curl -sSL https://raw.githubusercontent.com/Polymarket/polymarket-cli/main/install.sh | sh
```

### Compilar desde el código fuente

```bash
git clone https://github.com/Polymarket/polymarket-cli
cd polymarket-cli
cargo install --path .
```

## Inicio Rápido

```bash
# No necesitas wallet — explora mercados inmediatamente
polymarket markets list --limit 5
polymarket markets search "election"
polymarket events list --tag politics

# Consultar un mercado específico
polymarket markets get will-trump-win-the-2024-election

# Salida JSON para scripts
polymarket -o json markets list --limit 3
```

Para operar, configura una wallet:

```bash
polymarket setup
# O manualmente:
polymarket wallet create
polymarket approve set
```

## Configuración

### Configuración de Wallet

La CLI necesita una clave privada para firmar órdenes y transacciones on-chain. Tres formas de proporcionarla (verificadas en este orden):

1. **Flag CLI**: `--private-key 0xabc...`
2. **Variable de entorno**: `POLYMARKET_PRIVATE_KEY=0xabc...`
3. **Archivo de configuración**: `~/.config/polymarket/config.json`

```bash
# Crear una nueva wallet (genera clave aleatoria, guarda en config)
polymarket wallet create

# Importar una clave existente
polymarket wallet import 0xabc123...

# Ver la configuración actual
polymarket wallet show
```

El archivo de configuración (`~/.config/polymarket/config.json`):

```json
{
  "private_key": "0x...",
  "chain_id": 137,
  "signature_type": "proxy"
}
```

### Tipos de Firma

- `proxy` (por defecto) — usa el sistema de wallet proxy de Polymarket
- `eoa` — firma directamente con tu clave
- `gnosis-safe` — para wallets multisig

Sobrescribir por comando con `--signature-type eoa` o vía `POLYMARKET_SIGNATURE_TYPE`.

### Qué Necesita una Wallet

La mayoría de comandos funcionan sin wallet — explorar mercados, ver libros de órdenes, consultar precios. Solo necesitas wallet para:

- Crear y cancelar órdenes (`clob create-order`, `clob market-order`, `clob cancel-*`)
- Consultar balances e intercambios (`clob balance`, `clob trades`, `clob orders`)
- Operaciones on-chain (`approve set`, `ctf split/merge/redeem`)
- Gestión de recompensas y claves API (`clob rewards`, `clob create-api-key`)

## Formatos de Salida

Cada comando soporta `--output table` (por defecto) y `--output json`.

```bash
# Tabla legible para humanos (por defecto)
polymarket markets list --limit 2
```

```
 Question                            Price (Yes)  Volume   Liquidity  Status
 Will Trump win the 2024 election?   52.00¢       $145.2M  $1.2M      Active
 Will BTC hit $100k by Dec 2024?     67.30¢       $89.4M   $430.5K    Active
```

```bash
# JSON legible por máquina
polymarket -o json markets list --limit 2
```

```json
[
  { "id": "12345", "question": "Will Trump win the 2024 election?", "outcomePrices": ["0.52", "0.48"], ... },
  { "id": "67890", "question": "Will BTC hit $100k by Dec 2024?", ... }
]
```

Forma corta: `-o json` o `-o table`.

Los errores siguen el mismo patrón — modo tabla imprime `Error: ...` en stderr, modo JSON imprime `{"error": "..."}` en stdout. Código de salida no cero en ambos casos.

## Comandos

### Mercados

```bash
# Listar mercados con filtros
polymarket markets list --limit 10
polymarket markets list --active true --order volume_num
polymarket markets list --closed false --limit 50 --offset 25

# Obtener un mercado por ID o slug
polymarket markets get 12345
polymarket markets get will-trump-win

# Buscar
polymarket markets search "bitcoin" --limit 5

# Obtener etiquetas de un mercado
polymarket markets tags 12345
```

**Flags para `markets list`**: `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed`

### Eventos

Los eventos agrupan mercados relacionados (ej. "Elección 2024" contiene múltiples mercados sí/no).

```bash
polymarket events list --limit 10
polymarket events list --tag politics --active true
polymarket events get 500
polymarket events tags 500
```

**Flags para `events list`**: `--limit`, `--offset`, `--order`, `--ascending`, `--active`, `--closed`, `--tag`

### Etiquetas, Series, Comentarios, Perfiles, Deportes

```bash
# Etiquetas
polymarket tags list
polymarket tags get politics
polymarket tags related politics
polymarket tags related-tags politics

# Series (eventos recurrentes)
polymarket series list --limit 10
polymarket series get 42

# Comentarios en una entidad
polymarket comments list --entity-type event --entity-id 500
polymarket comments get abc123
polymarket comments by-user 0xf5E6...

# Perfiles públicos
polymarket profiles get 0xf5E6...

# Metadatos de deportes
polymarket sports list
polymarket sports market-types
polymarket sports teams --league NFL --limit 32
```

### Libro de Órdenes y Precios (CLOB)

Todo de solo lectura — no se necesita wallet.

```bash
# Verificar salud de la API
polymarket clob ok

# Precios
polymarket clob price 48331043336612883... --side buy
polymarket clob midpoint 48331043336612883...
polymarket clob spread 48331043336612883...

# Consultas por lotes (IDs de token separados por coma)
polymarket clob batch-prices "TOKEN1,TOKEN2" --side buy
polymarket clob midpoints "TOKEN1,TOKEN2"
polymarket clob spreads "TOKEN1,TOKEN2"

# Libro de órdenes
polymarket clob book 48331043336612883...
polymarket clob books "TOKEN1,TOKEN2"

# Último intercambio
polymarket clob last-trade 48331043336612883...

# Información del mercado
polymarket clob market 0xABC123...  # por condition ID
polymarket clob markets             # listar todos

# Historial de precios
polymarket clob price-history 48331043336612883... --interval 1d --fidelity 30

# Metadatos
polymarket clob tick-size 48331043336612883...
polymarket clob fee-rate 48331043336612883...
polymarket clob neg-risk 48331043336612883...
polymarket clob time
polymarket clob geoblock
```

**Opciones de intervalo para `price-history`**: `1m`, `1h`, `6h`, `1d`, `1w`, `max`

### Trading (CLOB, autenticado)

Requiere wallet configurada.

```bash
# Crear orden límite (comprar 10 acciones a $0.50)
polymarket clob create-order \
  --token 48331043336612883... \
  --side buy --price 0.50 --size 10

# Crear orden de mercado (comprar $5 de valor)
polymarket clob market-order \
  --token 48331043336612883... \
  --side buy --amount 5

# Publicar múltiples órdenes a la vez
polymarket clob post-orders \
  --tokens "TOKEN1,TOKEN2" \
  --side buy \
  --prices "0.40,0.60" \
  --sizes "10,10"

# Cancelar
polymarket clob cancel ORDER_ID
polymarket clob cancel-orders "ORDER1,ORDER2"
polymarket clob cancel-market --market 0xCONDITION...
polymarket clob cancel-all

# Ver tus órdenes e intercambios
polymarket clob orders
polymarket clob orders --market 0xCONDITION...
polymarket clob order ORDER_ID
polymarket clob trades

# Consultar balances
polymarket clob balance --asset-type collateral
polymarket clob balance --asset-type conditional --token 48331043336612883...
polymarket clob update-balance --asset-type collateral
```

**Tipos de orden**: `GTC` (por defecto), `FOK`, `GTD`, `FAK`. Añadir `--post-only` para órdenes límite.

### Recompensas y Claves API (CLOB, autenticado)

```bash
polymarket clob rewards --date 2024-06-15
polymarket clob earnings --date 2024-06-15
polymarket clob earnings-markets --date 2024-06-15
polymarket clob reward-percentages
polymarket clob current-rewards
polymarket clob market-reward 0xCONDITION...

# Verificar si las órdenes están obteniendo recompensas
polymarket clob order-scoring ORDER_ID
polymarket clob orders-scoring "ORDER1,ORDER2"

# Gestión de claves API
polymarket clob api-keys
polymarket clob create-api-key
polymarket clob delete-api-key

# Estado de la cuenta
polymarket clob account-status
polymarket clob notifications
polymarket clob delete-notifications "NOTIF1,NOTIF2"
```

### Datos On-Chain

Datos públicos — no se necesita wallet.

```bash
# Portafolio
polymarket data positions 0xWALLET_ADDRESS
polymarket data closed-positions 0xWALLET_ADDRESS
polymarket data value 0xWALLET_ADDRESS
polymarket data traded 0xWALLET_ADDRESS

# Historial de intercambios
polymarket data trades 0xWALLET_ADDRESS --limit 50

# Actividad
polymarket data activity 0xWALLET_ADDRESS

# Datos del mercado
polymarket data holders 0xCONDITION_ID
polymarket data open-interest 0xCONDITION_ID
polymarket data volume 12345  # event ID

# Tablas de clasificación
polymarket data leaderboard --period month --order-by pnl --limit 10
polymarket data builder-leaderboard --period week
polymarket data builder-volume --period month
```

### Aprobaciones de Contratos

Antes de operar, los contratos de Polymarket necesitan aprobaciones ERC-20 (USDC) y ERC-1155 (token CTF).

```bash
# Verificar aprobaciones actuales (solo lectura)
polymarket approve check
polymarket approve check 0xSOME_ADDRESS

# Aprobar todos los contratos (envía 6 transacciones on-chain, necesita MATIC para gas)
polymarket approve set
```

### Operaciones CTF

Dividir, fusionar y canjear tokens condicionales directamente on-chain.

```bash
# Dividir $10 USDC en tokens YES/NO
polymarket ctf split --condition 0xCONDITION... --amount 10

# Fusionar tokens de vuelta a USDC
polymarket ctf merge --condition 0xCONDITION... --amount 10

# Canjear tokens ganadores después de la resolución
polymarket ctf redeem --condition 0xCONDITION...

# Canjear posiciones de riesgo negativo
polymarket ctf redeem-neg-risk --condition 0xCONDITION... --amounts "10,5"

# Calcular IDs (solo lectura, no se necesita wallet)
polymarket ctf condition-id --oracle 0xORACLE... --question 0xQUESTION... --outcomes 2
polymarket ctf collection-id --condition 0xCONDITION... --index-set 1
polymarket ctf position-id --collection 0xCOLLECTION...
```

`--amount` está en USDC (ej., `10` = $10). El flag `--partition` por defecto es binario (`1,2`). Las operaciones on-chain requieren MATIC para gas en Polygon.

### Puente

Depositary activos de otras cadenas en Polymarket.

```bash
# Obtener direcciones de depósito (EVM, Solana, Bitcoin)
polymarket bridge deposit 0xWALLET_ADDRESS

# Listar cadenas y tokens soportados
polymarket bridge supported-assets

# Verificar estado del depósito
polymarket bridge status 0xDEPOSIT_ADDRESS
```

### Gestión de Wallet

```bash
polymarket wallet create               # Generar nueva wallet aleatoria
polymarket wallet create --force       # Sobrescribir existente
polymarket wallet import 0xKEY...      # Importar clave existente
polymarket wallet address              # Imprimir dirección de wallet
polymarket wallet show                 # Info completa de wallet (dirección, fuente, ruta de config)
polymarket wallet reset                # Borrar config (pide confirmación)
polymarket wallet reset --force        # Borrar sin confirmación
```

### Shell Interactivo

```bash
polymarket shell
# polymarket> markets list --limit 3
# polymarket> clob book 48331043336612883...
# polymarket> exit
```

Soporta historial de comandos. Todos los comandos funcionan igual que en CLI, solo sin el prefijo `polymarket`.

### Otros

```bash
polymarket status     # Verificación de salud de API
polymarket setup      # Asistente de configuración inicial guiado
polymarket upgrade    # Actualizar a la última versión
polymarket --version
polymarket --help
```

## Flujos de Trabajo Comunes

### Explorar e investigar mercados

```bash
polymarket markets search "bitcoin" --limit 5
polymarket markets get bitcoin-above-100k
polymarket clob book 48331043336612883...
polymarket clob price-history 48331043336612883... --interval 1d
```

### Configurar una nueva wallet y empezar a operar

```bash
polymarket wallet create
polymarket approve set                    # necesita MATIC para gas
polymarket clob balance --asset-type collateral
polymarket clob market-order --token TOKEN_ID --side buy --amount 5
```

### Monitorear tu portafolio

```bash
polymarket data positions 0xYOUR_ADDRESS
polymarket data value 0xYOUR_ADDRESS
polymarket clob orders
polymarket clob trades
```

### Crear y gestionar órdenes límite

```bash
# Crear orden
polymarket clob create-order --token TOKEN_ID --side buy --price 0.45 --size 20

# Verificarla
polymarket clob orders

# Cancelar si es necesario
polymarket clob cancel ORDER_ID

# O cancelar todo
polymarket clob cancel-all
```

### Script con salida JSON

```bash
# Enviar datos de mercado por pipe a jq
polymarket -o json markets list --limit 100 | jq '.[].question'

# Consultar precios programáticamente
polymarket -o json clob midpoint TOKEN_ID | jq '.mid'

# Manejo de errores en scripts
if ! result=$(polymarket -o json clob balance --asset-type collateral 2>/dev/null); then
  echo "Error al obtener balance"
fi
```

## Arquitectura

```
src/
  main.rs        -- Punto de entrada CLI, parsing clap, manejo de errores
  auth.rs        -- Resolución de wallet, proveedor RPC, autenticación CLOB
  config.rs      -- Archivo de configuración (~/.config/polymarket/config.json)
  shell.rs       -- REPL interactivo
  commands/      -- Un módulo por grupo de comandos
  output/        -- Renderizado de tabla y JSON por grupo de comandos
```

## Licencia

MIT
