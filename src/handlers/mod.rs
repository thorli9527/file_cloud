pub mod common;
mod download;
pub mod upload;

use crate::{ApiDoc, AppState};
use actix_web::web;
pub use common::*;
pub use download::*;
pub use upload::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    let api_doc = ApiDoc::openapi();
    common::configure(cfg);
    upload::configure(cfg, state.clone());
    download::configure(cfg, state.clone());
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone()),
    );
}
