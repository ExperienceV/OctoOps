# Contract of metric reports (JSONB)

Defines the shape of the `metrics` payload that agents (and any future device) send to the server: 
metrics grouped by category, each mapping to a numeric or boolean value.

```text
report = {
    "category_name": {
        "subcategory_name": number | bool
    }
}
```

## Example

```json
{
  "cpu":    { "percent": 12.5, "load_1": 0.15, "load_5": 0.20, "load_15": 0.18 },
  "memory": { "total_mb": 7900, "used_mb": 3100, "free_mb": 4800 },
  "disk":   { "total_gb": 80, "used_gb": 31 },
  "service":{ "running": true }
}
```
