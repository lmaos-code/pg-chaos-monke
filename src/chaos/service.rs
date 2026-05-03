use super::{ChaosStrategy, ColumnTarget};

/// Service Disruption Strategy
/// Randomly marks a specific day in the `calendar` table as `false`, canceling service for that route on that day.
pub struct ServiceDisruptionStrategy;

impl ChaosStrategy for ServiceDisruptionStrategy {
    fn name(&self) -> &'static str {
        "Service Day Cancellation"
    }

    fn can_apply(&self, target: &ColumnTarget) -> bool {
        let days = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
        target.table_name.to_lowercase() == "calendar" && days.contains(&target.column_name.to_lowercase().as_str())
    }

    fn generate_sql(&self, target: &ColumnTarget) -> String {
        format!(
            "UPDATE \"{}\" SET \"{}\" = NOT \"{}\" WHERE ctid IN (SELECT ctid FROM \"{}\" ORDER BY RANDOM() LIMIT 1)",
            target.table_name, target.column_name, target.column_name, target.table_name
        )
    }
}
