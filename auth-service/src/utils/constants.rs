use dotenvy::dotenv;
use lazy_static::lazy_static;

use crate::utils::constants::env::DATABASE_URL_ENV_VAR;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref DATABASE_URL: String = set_db_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_host_name();
}

fn set_token() -> String {
    dotenv().ok();

    let secret = std::env::var(env::JWT_SECRET_ENV_VAR)
        .expect("JWT_SECRET must be set in environment variables");

    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty");
    }
    secret
}

fn set_db_url() -> String {
    dotenv().ok();

    let db_url =
        std::env::var(DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set in environment variables");

    if db_url.is_empty() {
        panic!("DATABASE_URL must not be empty");
    }

    db_url
}

fn set_redis_host_name() -> String {
    dotenv().ok();
    std::env::var(env::REDIS_HOST_NAME_ENV_VAR)
      .unwrap_or(DEFAULT_REDIS_HOST_NAME.to_owned())
}

pub mod env {
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOST_NAME: &str = "127.0.0.1";

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}
