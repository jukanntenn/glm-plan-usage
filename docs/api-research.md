# GLM API 接口调研记录

## 接口信息
- **端点**: `/monitor/usage/quota/limit`
- **基础URL**: `https://open.bigmodel.cn/api`
- **认证方式**: Bearer Token (ANTHROPIC_AUTH_TOKEN)

## 实际 API 响应示例

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5,
        "number": 1,
        "usage": 100,
        "currentValue": 20,
        "remaining": 80,
        "percentage": 20,
        "usageDetails": [
          {
            "modelCode": "search-prime",
            "usage": 62
          },
          {
            "modelCode": "web-reader",
            "usage": 30
          },
          {
            "modelCode": "zread",
            "usage": 0
          }
        ]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3,
        "number": 5,
        "usage": 40000000,
        "currentValue": 13050812,
        "remaining": 26949188,
        "percentage": 32,
        "nextResetTime": 1770565089893
      }
    ]
  },
  "success": true
}
```

## 字段说明

### 通用字段
- `type`: 限额类型
  - `TIME_LIMIT`: MCP/工具调用次数限制
  - `TOKENS_LIMIT`: Token 使用量限制
- `usage`: 总量限制
- `currentValue`: 当前已使用量
- `remaining`: 剩余可用量
- `percentage`: 使用百分比 (0-100)

### TIME_LIMIT 特有字段
- `unit`: 单位代码 (5)
- `number`: 数量 (1)
- `usageDetails`: 各个 MCP 工具的使用详情
  - `modelCode`: 工具代码
  - `usage`: 该工具的使用次数

### TOKENS_LIMIT 特有字段
- `unit`: 单位代码 (3) - 可能表示时间单位
- `number`: 数量 (5) - 可能表示时间窗口大小
- `nextResetTime`: **下次重置时间戳（毫秒）** ⭐️ 重要

## 关键发现

### 1. 重置时间字段
- 字段名: `nextResetTime`
- 数据类型: 毫秒级时间戳 (i64)
- 单位: 毫秒 (需要除以 1000 转换为秒)
- **仅 TOKENS_LIMIT 返回此字段**
- TIME_LIMIT 不返回重置时间

### 2. Token 量级
- `usage`: 40,000,000 (40M tokens)
- `currentValue`: 13,050,812 (约 13M tokens)
- 显示格式建议: 使用 M (百万) 或 K (千) 为单位

### 3. MCP 计数
- `usage`: 100 (总次数)
- `currentValue`: 20 (已使用 20 次)
- `remaining`: 80 (剩余 80 次)
- 显示格式: 原始计数 (如 "M:20/100")

### 4. 时间窗口推断
- TOKENS_LIMIT: `unit=3, number=5` 可能表示 5 小时滚动窗口
- TIME_LIMIT: `unit=5, number=1` 可能表示 1 个自然月
- **需要进一步确认 `unit` 的具体含义**

## 设计影响

### 需要修改的数据结构

```rust
#[derive(Debug, Deserialize)]
pub struct QuotaLimitItem {
    #[serde(rename = "type")]
    pub quota_type: String,
    pub usage: i64,
    #[serde(rename = "currentValue")]
    pub current_value: i64,
    pub percentage: i32,
    // 新增字段
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>,
    #[serde(default)]
    pub unit: Option<i32>,
    #[serde(default)]
    pub number: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct QuotaUsage {
    pub used: i64,
    pub limit: i64,
    pub percentage: u8,
    pub time_window: String,
    // 新增字段
    pub reset_at: Option<i64>, // 秒级时间戳
}
```

### 倒计时计算注意事项
- `nextResetTime` 是毫秒时间戳，需要除以 1000 转换为秒
- 只有 TOKENS_LIMIT 有此字段，TIME_LIMIT 的 `reset_at` 为 None
- 时间戳示例: `1770565089893` (毫秒) → `1770565089` (秒)

## 测试时间戳
- 示例时间戳: `1770565089893` (毫秒)
- 转换后: `1770565089` (秒)
- 对应时间: 约 2026-02-08 (需要验证)

## 下一步行动
1. ✅ 确认 API 返回 `nextResetTime` 字段
2. ⏳ 更新 Rust 数据结构以支持该字段
3. ⏳ 实现倒计时计算逻辑（毫秒 → 秒转换）
4. ⏳ 更新格式化逻辑以显示倒计时
5. ⏳ 处理 TIME_LIMIT 无重置时间的边界情况
