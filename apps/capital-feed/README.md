# Capital.com Feed Service

Real-time market data feed from Capital.com for the Loom trading platform.

## Features

- **WebSocket Stream**: Real-time 1-minute candle updates
- **Auto-Reconnect**: Automatic reconnection with exponential backoff
- **Backfill**: Automatically fills gaps in historical data via REST API
- **Database Integration**: Stores candles directly to PostgreSQL

## Configuration

Create a `.env` file based on `.env.example`:

```bash
cp .env.example .env
```

Required environment variables:

- `CAPITAL_API_KEY` - Your Capital.com API key
- `CAPITAL_API_PASSWORD` - Your Capital.com API password  
- `DATABASE_URL` - PostgreSQL connection string
- `CAPITAL_SYMBOL` - Symbol to track (default: "NATURALGAS")
- `CAPITAL_SOURCE` - Source identifier for DB (default: "capitalcom")

## Running Locally

```bash
# Set environment variables
cp .env.example .env
# Edit .env with your credentials

# Run the service
cargo run
```

## Running with Docker

```bash
docker build -t capital-feed .
docker run --env-file .env capital-feed
```

## Deploying to Fly.io

1. **Create Fly app**:
```bash
fly apps create loom-capital-feed
```

2. **Set secrets**:
```bash
fly secrets set \
  CAPITAL_API_KEY="your-api-key" \
  CAPITAL_API_PASSWORD="your-password" \
  DATABASE_URL="postgresql://..." \
  CAPITAL_SYMBOL="NATURALGAS" \
  CAPITAL_SOURCE="capitalcom"
```

3. **Deploy** (use minimal resources for low cost):
```bash
fly deploy --ha=false
```

4. **Scale to shared-cpu-1x** (cheapest option):
```bash
fly scale vm shared-cpu-1x --memory 256
```

## Low-Budget Deployment

For minimal cost on Fly.io:

- **VM**: `shared-cpu-1x` (smallest)
- **Memory**: `256MB` (minimum for Rust app)
- **HA**: Disabled (`--ha=false`)
- **Region**: Single region only

Estimated cost: ~$2-3/month

## Architecture

```
Capital.com API (WebSocket)
         ↓
   capital-feed service
         ↓
   PostgreSQL (Loom DB)
         ↓
   Phoenix API
         ↓
   Frontend
```

## Data Flow

1. **On startup**: Check for data gaps and backfill via REST API
2. **Live stream**: Subscribe to 1m candle updates via WebSocket
3. **Store**: Upsert candles to `candles` table
4. **Reconnect**: Auto-reconnect on connection loss

## Troubleshooting

**Connection issues:**
- Verify API credentials are correct
- Check Capital.com API status
- Review logs: `fly logs` (on Fly.io) or `RUST_LOG=debug cargo run` (locally)

**Database errors:**
- Verify DATABASE_URL is correct
- Check PostgreSQL is accessible
- Ensure `candles` table exists (run migrations from Phoenix app)

**No data appearing:**
- Check symbol name matches Capital.com epic (e.g., "NATURALGAS")
- Verify WebSocket subscription is successful (check logs)
- Ensure database has correct schema

## License

MIT
