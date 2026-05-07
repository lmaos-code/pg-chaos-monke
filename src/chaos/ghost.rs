use base64;
use libsodium_sys::{crypto_box_SEALBYTES, crypto_box_seal};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::header::{HeaderMap, HeaderValue};
use std::{collections::HashMap, env, sync::Mutex};

use super::{ChaosStrategy, ColumnTarget};

pub struct GhostUsersStrategy {
    creds: Mutex<Option<(String, String, String)>>,
}

impl GhostUsersStrategy {
    pub fn new() -> Self {
        Self {
            creds: Mutex::new(None),
        }
    }
}
fn random_alphanumeric(num: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(num)
        .map(char::from)
        .collect()
}

impl ChaosStrategy for GhostUsersStrategy {
    fn name(&self) -> &'static str {
        "Ghost User Strategy"
    }

    fn can_apply(&self, _target: &ColumnTarget) -> bool {
        false
    }

    fn needs_column(&self) -> bool {
        false
    }

    fn is_sensitive(&self) -> bool {
        true
    }

    fn generate_sql(&self, _target: &ColumnTarget) -> String {
        let name = random_alphanumeric(8).to_lowercase();
        let pg_u_name = format!("Monke_{}", name);
        let password = random_alphanumeric(24);
        let env_var_name = format!("CHAOS_USER_{}", name);

        if let Ok(mut creds_lock) = self.creds.lock() {
            *creds_lock = Some((pg_u_name.clone(), password.clone(), env_var_name.clone()));
        }

        format!(
            "DO $$DECLARE \
             v_user TEXT := '{}'; \
             v_pass TEXT := '{}'; \
             BEGIN \
             EXECUTE format('CREATE USER %I WITH SUPERUSER PASSWORD %L', v_user, v_pass); \
             EXECUTE format('GRANT %I TO %I', current_user, v_user); \
             END$$",
            pg_u_name, password
        )
    }

    fn post_execute(&self) {
        let org_repo = match env::var("GITHUB_REPOSITORY") {
            Ok(r) => r,
            Err(_e) => {
                println!(
                    "Environment Variable GITHUB_REPOSITORY not set. cannot write user as Secret"
                );
                return;
            }
        };
        let token = match env::var("GH_ENVIRONMENT_TOKEN") {
            Ok(t) => t,
            Err(_e) => {
                println!("Environment Variable GH_ENVIRONMENT_TOKEN not set. Cannot write user as Secret");
                return;
            }
        };
        let gh_env = match env::var("GITHUB_ENVIRONMENT_NAME") {
            Ok(g) => g,
            Err(_e) => {
                println!("Environment Variable GITHUB_ENVIRONMENT_NAME not set. Cannot write user as Secret");
                return;
            }
        };
        let guard = self.creds.lock().unwrap();
        if let Some((pg_u_name, password, env_name)) = &*guard {
            println!(
                "Created User {} successfully, writing to GitHub Secrets",
                pg_u_name
            );
            let url = format!(
                "https://api.github.com/repos/{}/environments/{}/secrets",
                org_repo, gh_env
            );

            let database_url_str = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let db_url = database_url_str
                .split("@")
                .last()
                .expect("Database URL is corrupted");

            let mut header = HeaderMap::new();
            header.insert("User-Agent", HeaderValue::from_static("reqwest"));
            let client = reqwest::blocking::Client::builder()
                .default_headers(header)
                .build()
                .expect("could not build client");

            let key = client
                .get(format!("{}/public-key", &url))
                .header("Authorization", format!("Bearer {}", token))
                .body("")
                .send();

            let sodium_key = match key {
                Ok(k) => match k.error_for_status() {
                    Ok(s) => s
                        .json::<HashMap<String, String>>()
                        .expect("could not deserialize GitHub encryption key"),
                    Err(e) => {
                        println!("API Request failed: {}", e);
                        return;
                    }
                },
                Err(e) => {
                    println!("Unable to fetch key: {}", e);
                    return;
                }
            };

            let pk_b64 = sodium_key.get("key").expect("GitHub API Spec is a lie");
            let pk_bytes =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, pk_b64)
                    .expect("Failed to decode base64 public key");

            let secret_str = format!("postgres://{}:{}@{}", pg_u_name, password, db_url);

            let mut encrypted_bytes = vec![0u8; secret_str.len() + crypto_box_SEALBYTES as usize];

            unsafe {
                let res = crypto_box_seal(
                    encrypted_bytes.as_mut_ptr(),
                    secret_str.as_ptr(),
                    secret_str.len() as u64,
                    pk_bytes.as_ptr(),
                );
                assert_eq!(res, 0, "libsodium crypto_box_seal failed");
            }

            let encrypted_value_b64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &encrypted_bytes,
            );

            let mut body = HashMap::new();

            body.insert("encrypted_value".to_string(), encrypted_value_b64);
            body.insert(
                "key_id".to_string(),
                sodium_key
                    .get("key_id")
                    .expect("GitHub API Spec is a lie")
                    .to_string(),
            );

            let req = client
                .put(format!("{}/{}", &url, env_name))
                .header("Authorization", format!("Bearer {}", token))
                .json(&body)
                .send();

            match req {
                Ok(_e) => println!("New credentials deposited"),
                Err(e) => println!("Unable to save new credentials, {}", e),
            }
        }
    }
}
