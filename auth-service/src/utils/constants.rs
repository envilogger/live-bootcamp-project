use dotenvy::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
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

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
}

pub const JWT_COOKIE_NAME: &str = "jwt";

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}
