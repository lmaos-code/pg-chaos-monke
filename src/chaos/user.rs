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

        // Use a PL/pgSQL DO block with DECLARE variables so the actual values
        // are bound through PostgreSQL's own format() quoting (%I / %L) and
        // are never directly concatenated into executable SQL.
        // Dollar-quoting ($usr$ / $pwd$) safely embeds the alphanumeric
        // strings as PL/pgSQL text literals without risking SQL injection.
        format!(
            "DO $$DECLARE \
             v_user TEXT := $usr${}$usr$; \
             v_pass TEXT := $pwd${}$pwd$; \
             BEGIN \
             EXECUTE format('CREATE USER %%I WITH PASSWORD %%L', v_user, v_pass); \
             EXECUTE format('GRANT %%I TO %%I', current_user, v_user); \
             END$$",
            pg_user, password
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
                            // Strip any newlines from the password to prevent
                            // line-injection into the GITHUB_ENV key=value format.
                            let safe_password = password.replace(['\n', '\r'], "");
                            if writeln!(file, "{}={}", env_var_name, safe_password).is_ok() {
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
