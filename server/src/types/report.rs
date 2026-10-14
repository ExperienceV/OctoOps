use serde::{Deserialize, Serialize};

/// A leaf metric value: a JSON number or boolean (see `contracts/report.md`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Number(serde_json::Number),
    Bool(bool),
}

/// `report = { "category": { "subcategory": number | bool } }`
pub type Report = std::collections::HashMap<String, std::collections::HashMap<String, MetricValue>>;
