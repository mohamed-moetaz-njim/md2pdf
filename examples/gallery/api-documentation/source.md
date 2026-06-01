---
title: Payments API
subtitle: Reference v2
author: Developer Platform
---

# Overview

The Payments API is a JSON-over-HTTPS API. All requests require a bearer token and
are scoped to a single merchant account. Base URL: `https://api.example.com/v2`.

# Authentication

```http
GET /v2/charges HTTP/1.1
Host: api.example.com
Authorization: Bearer sk_live_********
```

# Endpoints

## Create a charge

`POST /v2/charges`

| Field      | Type    | Required | Description                          |
|:-----------|:--------|:--------:|:-------------------------------------|
| `amount`   | integer | yes      | Amount in the smallest currency unit |
| `currency` | string  | yes      | ISO 4217 code, e.g. `usd`            |
| `source`   | string  | yes      | A tokenized payment source           |
| `metadata` | object  | no       | Arbitrary key/value pairs            |

### Example

```bash
curl https://api.example.com/v2/charges \
  -H "Authorization: Bearer $KEY" \
  -d amount=1499 -d currency=usd -d source=tok_visa
```

```json
{
  "id": "ch_3Nf",
  "amount": 1499,
  "currency": "usd",
  "status": "succeeded"
}
```

## Errors

The API uses conventional HTTP status codes.

| Status | Meaning                | Notes                          |
|-------:|:-----------------------|:-------------------------------|
| 400    | `invalid_request`      | A parameter was missing/invalid|
| 401    | `authentication_error` | Bad or missing API key         |
| 429    | `rate_limited`         | Back off and retry             |

> **Idempotency:** send an `Idempotency-Key` header to safely retry POST requests.
