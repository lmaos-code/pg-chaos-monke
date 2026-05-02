use chrono::NaiveDate;
use sea_orm::schema::Schema;
use sea_orm::{
    ActiveValue::Set, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, Statement,
};

use crate::entity::{
    agency, calendar, calendardate, frequency, level, pathway, route, stop, stoptime, trip,
};

/// Generates dummy GTFS data to test the chaos monkey without affecting production data.
pub async fn generate_fixtures(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating fixtures for GTFS schema via SeaORM...");

    let schema = Schema::new(DbBackend::Postgres);

    // Helper to drop and create tables
    let backend = db.get_database_backend();

    // Drop tables if they exist to start fresh
    let drop_tables = vec![
        "DROP TABLE IF EXISTS stoptime CASCADE;",
        "DROP TABLE IF EXISTS pathway CASCADE;",
        "DROP TABLE IF EXISTS frequency CASCADE;",
        "DROP TABLE IF EXISTS calendardate CASCADE;",
        "DROP TABLE IF EXISTS trip CASCADE;",
        "DROP TABLE IF EXISTS route CASCADE;",
        "DROP TABLE IF EXISTS agency CASCADE;",
        "DROP TABLE IF EXISTS calendar CASCADE;",
        "DROP TABLE IF EXISTS stop CASCADE;",
        "DROP TABLE IF EXISTS level CASCADE;",
    ];
    for sql in drop_tables {
        db.execute(Statement::from_string(backend, sql.to_owned()))
            .await?;
    }

    // Create tables using SeaORM's schema builder in dependency order
    db.execute(backend.build(&schema.create_table_from_entity(agency::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(calendar::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(level::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(stop::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(route::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(calendardate::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(trip::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(stoptime::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(frequency::Entity)))
        .await?;
    db.execute(backend.build(&schema.create_table_from_entity(pathway::Entity)))
        .await?;

    // Seed Data using ActiveModels
    println!("Seeding Agency...");
    agency::Entity::insert(agency::ActiveModel {
        agency_id: Set("1".to_owned()),
        agency_name: Set(Some("Test Transit".to_owned())),
        agency_url: Set(Some("http://testtransit.com".to_owned())),
        agency_timezone: Set(Some("America/New_York".to_owned())),
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Seeding Calendar...");
    calendar::Entity::insert_many(vec![
        calendar::ActiveModel {
            service_id: Set("weekday_service".to_owned()),
            monday: Set(true),
            tuesday: Set(true),
            wednesday: Set(true),
            thursday: Set(true),
            friday: Set(true),
            saturday: Set(false),
            sunday: Set(false),
            start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        },
        calendar::ActiveModel {
            service_id: Set("weekend_service".to_owned()),
            monday: Set(false),
            tuesday: Set(false),
            wednesday: Set(false),
            thursday: Set(false),
            friday: Set(false),
            saturday: Set(true),
            sunday: Set(true),
            start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        },
    ])
    .exec(db)
    .await?;

    println!("Seeding Level...");
    level::Entity::insert(level::ActiveModel {
        level_id: Set("level1".to_owned()),
        level_index: Set(Some(rust_decimal::Decimal::from_f64(1.0).unwrap())),
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Seeding Stops...");
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;

    stop::Entity::insert_many(vec![
        stop::ActiveModel {
            stop_id: Set("stop1".to_owned()),
            stop_name: Set(Some("Central Station".to_owned())),
            stop_lat: Set(Some(Decimal::from_f64(40.7128).unwrap())),
            stop_lon: Set(Some(Decimal::from_f64(-74.0060).unwrap())),
            location_type: Set(Some(0)),
            level_id: Set(Some("level1".to_owned())),
            ..Default::default()
        },
        stop::ActiveModel {
            stop_id: Set("stop2".to_owned()),
            stop_name: Set(Some("North Park".to_owned())),
            stop_lat: Set(Some(Decimal::from_f64(40.7306).unwrap())),
            stop_lon: Set(Some(Decimal::from_f64(-73.9866).unwrap())),
            location_type: Set(Some(0)),
            level_id: Set(Some("level1".to_owned())),
            ..Default::default()
        },
        stop::ActiveModel {
            stop_id: Set("stop3".to_owned()),
            stop_name: Set(Some("West End".to_owned())),
            stop_lat: Set(Some(Decimal::from_f64(40.7411).unwrap())),
            stop_lon: Set(Some(Decimal::from_f64(-73.9897).unwrap())),
            location_type: Set(Some(0)),
            level_id: Set(Some("level1".to_owned())),
            ..Default::default()
        },
    ])
    .exec(db)
    .await?;

    println!("Seeding Route...");
    route::Entity::insert(route::ActiveModel {
        route_id: Set("route1".to_owned()),
        agency_id: Set(Some("1".to_owned())),
        route_short_name: Set(Some("R1".to_owned())),
        route_long_name: Set(Some("Red Line".to_owned())),
        route_type: Set(Some(1)), // Subway
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Seeding CalendarDate...");
    calendardate::Entity::insert(calendardate::ActiveModel {
        service_id: Set("weekday_service".to_owned()),
        date: Set(NaiveDate::from_ymd_opt(2026, 7, 4).unwrap()), // Holiday exception
        exception_type: Set(2), // Removed
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Seeding Trip...");
    trip::Entity::insert(trip::ActiveModel {
        route_id: Set(Some("route1".to_owned())),
        service_id: Set(Some("weekday_service".to_owned())),
        trip_id: Set("trip1".to_owned()),
        trip_headsign: Set(Some("West End".to_owned())),
        direction_id: Set(Some(0)),
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Seeding Stoptimes...");
    stoptime::Entity::insert_many(vec![
        stoptime::ActiveModel {
            trip_id: Set("trip1".to_owned()),
            stop_sequence: Set(1),
            stop_id: Set(Some("stop1".to_owned())),
            arrival_time: Set(Some(28800)),
            departure_time: Set(Some(28800)),
            ..Default::default()
        },
        stoptime::ActiveModel {
            trip_id: Set("trip1".to_owned()),
            stop_sequence: Set(2),
            stop_id: Set(Some("stop2".to_owned())),
            arrival_time: Set(Some(29400)),
            departure_time: Set(Some(29400)),
            ..Default::default()
        },
        stoptime::ActiveModel {
            trip_id: Set("trip1".to_owned()),
            stop_sequence: Set(3),
            stop_id: Set(Some("stop3".to_owned())),
            arrival_time: Set(Some(30000)),
            departure_time: Set(Some(30000)),
            ..Default::default()
        },
    ])
    .exec(db)
    .await?;

    println!("Seeding Frequency...");
    frequency::Entity::insert(frequency::ActiveModel {
        trip_id: Set("trip1".to_owned()),
        start_time: Set("08:00:00".to_owned()),
        end_time: Set(Some("10:00:00".to_owned())),
        headway_secs: Set(600), // 10 minutes
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Seeding Pathway...");
    pathway::Entity::insert(pathway::ActiveModel {
        pathway_id: Set("pathway1".to_owned()),
        from_stop_id: Set(Some("stop1".to_owned())),
        to_stop_id: Set(Some("stop2".to_owned())),
        pathway_mode: Set(Some(1)), // Walkway
        is_bidirectional: Set(Some(true)),
        traversal_time: Set(Some(300)), // 5 mins walking
        ..Default::default()
    })
    .exec(db)
    .await?;

    println!("Fixtures generated successfully.");
    Ok(())
}
