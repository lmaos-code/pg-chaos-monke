use super::{ChaosStrategy, ColumnTarget};

/// XSS Injection Strategy
/// Targets text/varchar fields and replaces their content with a malicious script.
pub struct JavascriptInjectionStrategy;

impl ChaosStrategy for JavascriptInjectionStrategy {
    fn name(&self) -> &'static str {
        "JavaScript XSS Injection"
    }

    fn can_apply(&self, target: &ColumnTarget) -> bool {
        let col = target.column_name.to_lowercase();
        // Specifically target names, urls, descriptions
        let is_text_field = col.contains("name")
            || col.contains("url")
            || col.contains("desc")
            || col.contains("headsign");
        let dt = target.data_type.to_lowercase();
        is_text_field && (dt.contains("char") || dt.contains("text"))
    }

    fn generate_sql(&self, target: &ColumnTarget) -> String {
        format!(
            "UPDATE \"{}\" SET \"{}\" = '<script>eval(atob(\"YWxlcnQoJ01vbmtlIGdldCBCYW5hbmEuIFlvdSBoYXZlIGJlZW4gcHduZCBieSB5b3UgZmF2b3JpdGUgQ2hhb3MgTW9ua2V5Jyk7Cg==\"))</script>' WHERE ctid IN (SELECT ctid FROM \"{}\" ORDER BY RANDOM() LIMIT 70)",
            target.table_name, target.column_name, target.table_name
        )
    }
}
