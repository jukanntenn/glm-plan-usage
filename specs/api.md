# API Integration

> GLM platform API endpoints, response formats, and field documentation.

## Endpoint

`GET /monitor/usage/quota/limit` with Bearer token from `ANTHROPIC_AUTH_TOKEN`.

| Platform | URL Pattern              | Example Base URL                         | Monitoring Endpoint                                      |
| -------- | ------------------------ | ---------------------------------------- | -------------------------------------------------------- |
| Zhipu    | `bigmodel.cn` or `zhipu` | `https://open.bigmodel.cn/api/anthropic` | `https://open.bigmodel.cn/api/monitor/usage/quota/limit` |
| Zai      | `api.z.ai`               | `https://api.z.ai/api/paas/v4/`          | Platform-specific mapping                                |

The base URL is derived from `ANTHROPIC_BASE_URL`; URL transformation to the monitoring endpoint is platform-specific.

## Request

```http
GET /monitor/usage/quota/limit HTTP/1.1
Host: open.bigmodel.cn
Authorization: Bearer <ANTHROPIC_AUTH_TOKEN>
```

Retry: 3 attempts with 100ms delay between each.

## Response Format

```json
{
  "code": 200,
  "msg": "Success",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5,
        "number": 1,
        "usage": 100,
        "currentValue": 28,
        "remaining": 72,
        "percentage": 28,
        "nextResetTime": 1772615765983,
        "usageDetails": [
          { "modelCode": "search-prime", "usage": 67 },
          { "modelCode": "web-reader", "usage": 33 }
        ]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3,
        "number": 5,
        "percentage": 1,
        "nextResetTime": 1771073738808
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 6,
        "usage": 500000,
        "currentValue": 120000,
        "percentage": 24,
        "nextResetTime": 1744137600000
      }
    ],
    "level": "lite"
  },
  "success": true
}
```

## Quota Types

### TIME_LIMIT — MCP/Tool Call Quota

| Field             | Type    | Description                                      |
| ----------------- | ------- | ------------------------------------------------ |
| `usage`           | `i64`   | Total call limit (count)                         |
| `currentValue`    | `i64`   | Current usage count                              |
| `remaining`       | `i64`   | Remaining count                                  |
| `percentage`      | `i32`   | Usage percentage (0-100)                         |
| `nextResetTime`   | `i64`   | Reset timestamp (milliseconds)                   |
| `usageDetails`    | `Array` | Per-tool usage breakdown (`modelCode` + `usage`) |
| `unit` / `number` | `i64`   | Time window: unit=5, number=1 → 1 calendar month |

### TOKENS_LIMIT (unit=3) — 5-Hour Rolling Token Quota

| Field             | Type  | Description                                    |
| ----------------- | ----- | ---------------------------------------------- |
| `percentage`      | `i32` | Usage percentage (0-100)                       |
| `nextResetTime`   | `i64` | Reset timestamp (milliseconds)                 |
| `unit` / `number` | `i64` | Time window: unit=3, number=5 → 5-hour rolling |

### TOKENS_LIMIT (unit=6) — Weekly Token Quota

Only returned for new plan users. Absent for legacy plans (code handles as `None`).

| Field           | Type          | Description                                   |
| --------------- | ------------- | --------------------------------------------- |
| `usage`         | `i64`         | Total weekly token limit                      |
| `currentValue`  | `i64`         | Tokens used this week                         |
| `percentage`    | `i32`         | Usage percentage (0-100)                      |
| `nextResetTime` | `Option<i64>` | Reset timestamp (milliseconds), may be `null` |
| `unit`          | `i64`         | Fixed value `6` (weekly)                      |

## Timestamp Handling

- `nextResetTime` is a **millisecond** timestamp — divide by 1000 for seconds
- Countdown format: `⌛️ HH:MM` (e.g., "⌛️ 1:44" = 1 hour 44 minutes)
- Edge cases: Expired → `⌛️ 0:00`, Missing → `⌛️ --:--`
