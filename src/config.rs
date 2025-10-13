use tera::Tera;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        Tera::new("src/templates/**/*").expect("Template loading failed")
    };

    pub static ref IS_DEV: bool = {
        env::var("RUST_ENV").unwrap_or("development".into()) == "development"
    };
}
