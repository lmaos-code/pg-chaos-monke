use std::io::Write;
use std::sync::Mutex;

use rand::distributions::Alphanumeric;
use rand::Rng;

use super::{ChaosStrategy, ColumnTarget};

fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// Ghost User Injection Strategy
/// Creates a new PostgreSQL user that inherits the current session user's role,
/// then writes the generated password to the GitHub Actions environment file
/// (`GITHUB_ENV`) under a randomly chosen variable name.
pub struct GhostUserStrategy {
    // (pg_username, password, env_var_name) stored after generate_sql is called
    creds: Mutex<Option<(String, String, String)>>,
}

impl GhostUserStrategy {
    pub fn new() -> Self {
        Self {
            creds: Mutex::new(None),
        }
    }
}

impl ChaosStrategy for GhostUserStrategy {
    fn name(&self) -> &'static str {
        "Ghost User Injection"
    }

    fn needs_column(&self) -> bool {
        false
    }

    fn can_apply(&self, _target: &ColumnTarget) -> bool {
        false
    }

    fn generate_sql(&self, _target: &ColumnTarget) -> String {
        let pg_user = format!("chaos_{}", random_alphanumeric(8).to_lowercase());
        let password = random_alphanumeric(24);
        let env_var_name = format!("CHAOS_{}", random_alphanumeric(8).to_uppercase());

        // Store credentials so post_execute can export them.
        *self.creds.lock().unwrap() = Some((pg_user.clone(), password.clone(), env_var_name));

        // Use a PL/pgSQL DO block so both statements execute atomically.
        // %I quotes an identifier, %L quotes a string literal.
        format!(
            "DO $$ BEGIN \
             EXECUTE format('CREATE USER %I WITH PASSWORD %L', '{}', '{}'); \
             EXECUTE format('GRANT %I TO %I', current_user, '{}'); \
             END $$",
            pg_user, password, pg_user
        )
    }

    fn post_execute(&self) {
        let guard = self.creds.lock().unwrap();
        if let Some((pg_user, password, env_var_name)) = &*guard {
            println!(
                "Ghost user '{}' created. Writing password to GITHUB_ENV as '{}'.",
                pg_user, env_var_name
            );
            match std::env::var("GITHUB_ENV") {
                Ok(env_file_path) => {
                    match std::fs::OpenOptions::new()
                        .append(true)
                        .open(&env_file_path)
                    {
                        Ok(mut file) => {
                            if writeln!(file, "{}={}", env_var_name, password).is_ok() {
                                println!(
                                    "Password successfully written to GITHUB_ENV as '{}'.",
                                    env_var_name
                                );
                            } else {
                                println!("Failed to write to GITHUB_ENV file.");
                            }
                        }
                        Err(e) => println!("Failed to open GITHUB_ENV file: {}", e),
                    }
                }
                Err(_) => println!(
                    "GITHUB_ENV is not set; skipping environment variable export."
                ),
            }
        }
    }
}
