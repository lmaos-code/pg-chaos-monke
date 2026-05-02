use super::{ChaosStrategy, ColumnTarget};

/// Time Confusion Strategy
/// Adds a massive delay to arrival or departure times in `stoptime` table.
pub struct TimeConfusionStrategy;

impl ChaosStrategy for TimeConfusionStrategy {
    fn name(&self) -> &'static str {
        "Massive Delay Injection"
    }

    fn can_apply(&self, target: &ColumnTarget) -> bool {
        target.table_name.to_lowercase() == "stoptime"
            && (target.column_name.to_lowercase() == "arrival_time"
                || target.column_name.to_lowercase() == "departure_time")
    }

    fn generate_sql(&self, target: &ColumnTarget) -> String {
        // GTFS arrival_time/departure_time are usually strings like "HH:MM:SS" or integers representing seconds from midnight.
        // The generated entities map them to `i32` or similar (in this schema, they are i32).
        // Let's add an arbitrary number of seconds (e.g. 5 hours = 18000 seconds)
        format!(
            "UPDATE \"{}\" SET \"{}\" = \"{}\" + (RANDOM() * 18000)::int WHERE ctid IN (SELECT ctid FROM \"{}\" ORDER BY RANDOM() LIMIT 1)",
            target.table_name, target.column_name, target.column_name, target.table_name
        )
    }
}
