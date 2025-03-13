pub mod common;
pub mod upload;

use crate::AppState;
use actix_web::web;
pub use common::*;
pub use upload::*;

pub fn configure(cfg: &mut web::ServiceConfig,state: web::Data<AppState>) {
    common::configure(cfg);
    upload::configure(cfg,state);
}