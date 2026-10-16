package types

// Report matches contracts/report.md:
//
//	report = { "category": { "subcategory": number | bool } }
//
// A leaf value is either a JSON number or a bool.
type Report map[string]map[string]any
