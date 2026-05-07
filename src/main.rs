mod chaos;
mod entity;
mod fixtures;

use chaos::geo::GeoChaosStrategy;
use chaos::ghost::GhostUsersStrategy;
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
        Box::new(GhostUsersStrategy::new()),
    ];

    // Pick a random valid move
    let mut rng = rand::thread_rng();
    let selected_strategy = strategies
        .choose(&mut rng)
        .expect("Could not select a strategy");

    let dummy_column = ColumnTarget {
        table_name: String::new(),
        column_name: String::new(),
        data_type: String::new(),
    };

    let mut target: &ColumnTarget = &dummy_column.clone();
    let mut valid_columns = Vec::new();

    if selected_strategy.needs_column() {
        for column in &columns {
            if selected_strategy.can_apply(column) {
                valid_columns.push(column.clone());
            }
        }
        target = valid_columns
            .choose(&mut rng)
            .expect("Could not select valid colum for strategy")
    }

    println!("Spinning the wheel...");
    println!("Selected Strategy : {}", selected_strategy.name());
    println!("Target Table      : {}", target.table_name);
    println!(
        "Target Column     : {} ({})",
        target.column_name, target.data_type
    );

    let chaos_sql = selected_strategy.generate_sql(target);

    if selected_strategy.is_sensitive() {
        println!("Executing SQL     : [REDACTED]");
    } else {
        println!("Executing SQL     : {}", chaos_sql);
    }

    // Execute the chaos
    match db
        .execute(Statement::from_string(DbBackend::Postgres, chaos_sql))
        .await
    {
        Ok(exec_res) => {
            println!(
                "Chaos successfully injected. Rows affected: {}",
                exec_res.rows_affected()
            );
            tokio::task::block_in_place(|| {
                selected_strategy.post_execute();
            });
        }
        Err(e) => println!("Failed to inject chaos: {}", e),
    }

    Ok(())
}
