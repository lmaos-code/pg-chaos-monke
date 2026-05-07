use std::fmt::Debug;

pub mod geo;
pub mod ghost;
pub mod service;
pub mod time;
pub mod xss;

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnTarget {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
}

pub trait ChaosStrategy: Send + Sync {
    /// Name of the strategy for logging.
    fn name(&self) -> &'static str;

    /// Determines if this strategy can be applied to the given column/table.
    fn can_apply(&self, target: &ColumnTarget) -> bool;

    /// Determinies if this strategy required a specifc column
    fn needs_column(&self) -> bool {
        true
    }

    /// Generates the SQL to execute the chaos on a single random row.
    fn generate_sql(&self, target: &ColumnTarget) -> String;

    /// Indicates if the generated SQL contains sensitive information like passwords.
    fn is_sensitive(&self) -> bool {
        false
    }

    fn post_execute(&self) {}
}
