use std::fmt::Debug;

pub mod geo;
pub mod service;
pub mod time;
pub mod user;
pub mod xss;

#[derive(Debug, Clone)]
pub struct ColumnTarget {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
}

pub trait ChaosStrategy: Send + Sync {
    /// Name of the strategy for logging.
    fn name(&self) -> &'static str;

    /// Whether this strategy requires a column target.
    /// Strategies that return `false` here are added once to the pool
    /// with a dummy target and their `generate_sql` / `post_execute` must
    /// not rely on that target.
    fn needs_column(&self) -> bool {
        true
    }

    /// Determines if this strategy can be applied to the given column/table.
    fn can_apply(&self, target: &ColumnTarget) -> bool;

    /// Generates the SQL to execute the chaos.
    fn generate_sql(&self, target: &ColumnTarget) -> String;

    /// Optional hook called after successful SQL execution.
    fn post_execute(&self) {}
}
