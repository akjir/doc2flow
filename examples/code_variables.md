---
title: "Microservice Deployment & Infrastructure Setup"
company: "Acme Cloud Ops"
version: "1.0.0"
date: "2026-08-02"
language: "en"
---

[Variables]
| Variable | Value |
| --- | --- |
| SYSTEM | prod-server |
| PORT | 8080 |
| DB_HOST | db.internal.net |
| APP_ENV | production |
| UNUSED_METRIC_PORT | 9090 |

# Microservice Deployment Procedure

This document outlines the automated deployment workflow for our cloud microservice stack. Dynamic variable substitution replaces placeholders matching `{{VARIABLE_NAME}}` when copying code snippets to your clipboard.

### Initial Environment Verification

Verify connectivity to the primary application server before beginning deployment:

```bash
ping -c 3 {{SYSTEM}}.local
```

## Database Initialization & Migration

Check the readiness of the primary database host:

```bash
pg_isready -h {{DB_HOST}} -p 5432 -U postgres
```

Configure the application database instance environment settings:

```sql
ALTER DATABASE app_db SET configuration.environment = '{{APP_ENV}}';
```

## Service Container Deployment

Spin up the API microservice container on the designated port:

```bash
docker run -d --name api-service -e ENV={{APP_ENV}} -p {{PORT}}:8080 acme/api-service:latest
```

Perform an automated health check against the newly deployed service:

```bash
curl -f -H "Authorization: Bearer {{AUTH_TOKEN}}" https://{{SYSTEM}}.local:{{PORT}}/api/v1/health
```

