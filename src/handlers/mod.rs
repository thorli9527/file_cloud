pub mod common;
pub mod upload;

use actix_web::web;
pub use common::*;
pub use upload::*;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig,state: web::Data<AppState>) {
    common::configure(cfg);
    upload::configure(cfg,state);
}