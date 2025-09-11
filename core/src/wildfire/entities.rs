// use uuid::Uuid;
//
// use chrono::{DateTime, Utc};
// use sqlx::FromRow;

// #[derive(Debug, FromRow)]
// pub struct WildfireSimulation {
//     pub id: i64,
//     pub started_at: DateTime<Utc>,
//     pub parameters: Option<String>,
// }
//
// #[derive(Debug, FromRow)]
// pub struct WildfireTimestep {
//     pub id: i64,
//     pub simulation_id: i64,
//     pub step_number: i64,
// }
//
// #[derive(Debug, FromRow)]
// pub struct WildfireAgentLog {
//     pub id: i64,
//     pub timestep_id: i64,
//     pub agent_id: i64,
//     pub x: f64,
//     pub y: f64,
//     pub power: Option<f64>,
//     pub suppressant: Option<f64>,
//     pub capacity: Option<f64>,
//     pub equipment: Option<String>,
// }
//
// #[derive(Debug, FromRow)]
// pub struct WildfireFireLog {
//     pub id: i64,
//     pub timestep_id: i64,
//     pub fire_id: i64,
//     pub x: f64,
//     pub y: f64,
//     pub power: Option<f64>,
//     pub suppressant: Option<f64>,
//     pub capacity: Option<f64>,
//     pub equipment: Option<String>,
// }
//
// #[derive(Debug, FromRow)]
// pub struct WildfireTileFuel {
//     pub id: i64,
//     pub timestep_id: i64,
//     pub x: f64,
//     pub y: f64,
//     pub fuel: f64,
// }
