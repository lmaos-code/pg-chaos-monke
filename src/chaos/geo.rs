use super::{ChaosStrategy, ColumnTarget};

/// Geographic Chaos Strategy
/// Targets the `stop` table and randomizes the `stop_lat` or `stop_lon` to valid coordinates.
pub struct GeoChaosStrategy;

impl ChaosStrategy for GeoChaosStrategy {
    fn name(&self) -> &'static str {
        "Random Geographic Coordinate"
    }

    fn can_apply(&self, target: &ColumnTarget) -> bool {
        target.table_name.to_lowercase() == "stop"
            && (target.column_name.to_lowercase() == "stop_lat"
                || target.column_name.to_lowercase() == "stop_lon")
    }

    fn needs_column(&self) -> bool {
        true
    }

    fn generate_sql(&self, target: &ColumnTarget) -> String {
        // Latitude is -90 to 90, Longitude is -180 to 180
        let is_lat = target.column_name.to_lowercase() == "stop_lat";
        let max_val = if is_lat { 90 } else { 180 };

        format!(
            "UPDATE \"{}\" SET \"{}\" = (RANDOM() * {} * 2) - {} WHERE ctid IN (SELECT ctid FROM \"{}\" ORDER BY RANDOM() LIMIT 300)",
            target.table_name, target.column_name, max_val, max_val, target.table_name
        )
    }
}
