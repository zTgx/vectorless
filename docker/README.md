# Docker Deployment Guide

This guide covers deploying vectorless using Docker and Docker Compose.

## Prerequisites

- Docker 20.10 or later
- Docker Compose 2.0 or later
- At least 1GB of available disk space
- ZAI API key (or compatible LLM endpoint)

## Quick Start

### 1. Create Environment File

Create a `.env` file in the project root:

```bash
# LLM Configuration
ZAI_API_KEY=your-api-key-here
ZAI_ENDPOINT=https://api.z.ai/api/paas/v4
ZAI_MODEL=glm-5

# Service Configuration
SERVICE_HOST=0.0.0.0
SERVICE_PORT=8080

# API Authentication (optional, comma-separated keys)
# SERVICE_API_KEYS=key1,key2,key3

# Logging
RUST_LOG=info
```

### 2. Start the Service

```bash
# From project root
docker compose -f docker/docker-compose.yml up -d
```

The service will be available at `http://localhost:8080`

### 3. Check Health Status

```bash
curl http://localhost:8080/health
```

Expected response:
```json
{"status":"ok"}
```

## Building Images

### Build Service Image

```bash
docker build -f docker/Dockerfile.service -t vectorless-service:latest .
```

### Build CLI Image

```bash
docker build -f docker/Dockerfile.cli -t vectorless-cli:latest .
```

## Docker Compose Services

### vectorless (Service)

Main HTTP service providing REST API for document indexing and RAG queries.

**Environment Variables:**

| Variable | Description | Default |
|----------|-------------|---------|
| `ZAI_API_KEY` | LLM API key | *required* |
| `ZAI_ENDPOINT` | LLM endpoint URL | `https://api.z.ai/api/paas/v4` |
| `ZAI_MODEL` | Model name | `glm-5` |
| `SERVICE_HOST` | Bind address | `0.0.0.0` |
| `SERVICE_PORT` | Bind port | `8080` |
| `SERVICE_DATA_DIR` | Data directory | `/data` |
| `SERVICE_API_KEYS` | API authentication keys | *empty (disabled)* |
| `RUST_LOG` | Log level | `info` |

**Volumes:**
- `vectorless-data:/data` - Persistent data storage

**Ports:**
- `8080:8080` - HTTP API

### vectorless-cli (CLI)

Command-line interface for ad-hoc operations.

**Usage:**

```bash
# Start CLI container
docker compose -f docker/docker-compose.yml --profile cli up vectorless-cli

# In another terminal, run commands
docker compose -f docker/docker-compose.yml --profile cli exec vectorless-cli vectorless --help
```

## Container Management

### View Logs

```bash
# Service logs
docker compose -f docker/docker-compose.yml logs -f vectorless

# CLI logs
docker compose -f docker/docker-compose.yml logs -f vectorless-cli
```

### Stop Services

```bash
docker compose -f docker/docker-compose.yml down
```

### Remove Data Volume

```bash
docker compose -f docker/docker-compose.yml down -v
```

### Restart Services

```bash
docker compose -f docker/docker-compose.yml restart
```

## API Usage

### Create Document

```bash
curl -X POST http://localhost:8080/documents \
  -H "Content-Type: application/json" \
  -d '{"title": "My Document"}'
```

### List Documents

```bash
curl http://localhost:8080/documents
```

### Query Documents

```bash
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is this document about?"}'
```

### Health Check

```bash
curl http://localhost:8080/health
```

## Production Deployment

### Security Considerations

1. **API Authentication**: Set `SERVICE_API_KEYS` with comma-separated keys
2. **HTTPS**: Use a reverse proxy (nginx/traefik) for SSL termination
3. **Network**: Use private Docker networks for inter-service communication
4. **Updates**: Pin specific image tags in production

### Resource Limits

Add to `docker-compose.yml`:

```yaml
services:
  vectorless:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M
```

### Reverse Proxy Configuration

Example nginx configuration:

```nginx
server {
    listen 443 ssl http2;
    server_name vectorless.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Troubleshooting

### Container Won't Start

Check logs:
```bash
docker compose -f docker/docker-compose.yml logs vectorless
```

Common issues:
- Missing `ZAI_API_KEY`
- Port 8080 already in use
- Insufficient disk space

### Health Check Failing

```bash
# Check container status
docker ps

# Execute health check manually
docker exec vectorless-service curl -f http://localhost:8080/health
```

### Data Persistence

Data is stored in the `vectorless-data` volume. To backup:

```bash
docker run --rm -v vectorless-data:/data -v $(pwd):/backup \
  debian:bookworm-slim tar czf /backup/vectorless-backup.tar.gz /data
```

To restore:

```bash
docker run --rm -v vectorless-data:/data -v $(pwd):/backup \
  debian:bookworm-slim tar xzf /backup/vectorless-backup.tar.gz -C /
```

## Advanced Configuration

### Custom Indexer Configuration

```bash
docker compose -f docker/docker-compose.yml up -d \
  -e SERVICE_SUBSECTION_THRESHOLD=500 \
  -e SERVICE_MAX_SEGMENT_TOKENS=6000 \
  -e SERVICE_MAX_SUMMARY_TOKENS=300
```

### Multiple Instances

```bash
# Create multiple service instances
docker compose -f docker/docker-compose.yml up -d --scale vectorless=3
```

Note: Each instance will need unique port mappings.
