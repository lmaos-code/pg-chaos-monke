mod chaos;
mod entity;
mod fixtures;

use chaos::geo::GeoChaosStrategy;
use chaos::service::ServiceDisruptionStrategy;
use chaos::time::TimeConfusionStrategy;
use chaos::xss::JavascriptInjectionStrategy;
use chaos::{ChaosStrategy, ColumnTarget};

use dotenvy::dotenv;
use rand::seq::SliceRandom;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to PostgreSQL
    let db: DatabaseConnection = Database::connect(&database_url).await?;

    // Seed fixtures if requested
    if env::var("SEED_FIXTURES").unwrap_or_default() == "true" {
        fixtures::generate_fixtures(&db).await?;
        exit(0);
    }

    // Introspect the database natively via information_schema
    let discover_query = Statement::from_string(
        DbBackend::Postgres,
        r#"
        SELECT table_name, column_name, data_type 
        FROM information_schema.columns 
        WHERE table_schema = 'public'
          AND is_updatable = 'YES'
        "#
        .to_owned(),
    );

    let query_results = db.query_all(discover_query).await?;

    let mut columns = Vec::new();
    for row in query_results {
        let table_name: String = row.try_get("", "table_name")?;
        let column_name: String = row.try_get("", "column_name")?;
        let data_type: String = row.try_get("", "data_type")?;

        columns.push(ColumnTarget {
            table_name,
            column_name,
            data_type,
        });
    }

    if columns.is_empty() {
        println!("No updatable columns found in the public schema.");
        return Ok(());
    }

    // Register all strategies
    let strategies: Vec<Box<dyn ChaosStrategy>> = vec![
        Box::new(JavascriptInjectionStrategy),
        Box::new(GeoChaosStrategy),
        Box::new(ServiceDisruptionStrategy),
        Box::new(TimeConfusionStrategy),
    ];

    // Build a list of valid (Strategy, Target) pairs
    let mut valid_moves = Vec::new();
    for strategy in &strategies {
        for column in &columns {
            if strategy.can_apply(column) {
                valid_moves.push((strategy, column.clone()));
            }
        }
    }

    if valid_moves.is_empty() {
        println!("No chaos strategies are applicable to the current database schema.");
        return Ok(());
    }

    // Pick a random valid move
    let mut rng = rand::thread_rng();
    let (selected_strategy, target) = valid_moves.choose(&mut rng).expect("Failed to select move");

    println!("Spinning the wheel...");
    println!("Selected Strategy : {}", selected_strategy.name());
    println!("Target Table      : {}", target.table_name);
    println!(
        "Target Column     : {} ({})",
        target.column_name, target.data_type
    );

    let chaos_sql = selected_strategy.generate_sql(&target);
    println!("Executing SQL     : {}", chaos_sql);

    // Execute the chaos
    match db
        .execute(Statement::from_string(DbBackend::Postgres, chaos_sql))
        .await
    {
        Ok(exec_res) => println!(
            "Chaos successfully injected. Rows affected: {}",
            exec_res.rows_affected()
        ),
        Err(e) => println!("Failed to inject chaos: {}", e),
    }

    Ok(())
}
