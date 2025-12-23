#!/bin/bash

docker compose -f docker-compose.dev.yml up -d

echo "DATABASE_URL=postgres://user:pass@localhost:5433/flora"
echo "REDIS_URL=redis://localhost:5434"