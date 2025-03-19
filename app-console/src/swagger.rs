
use utoipa::OpenApi;
use crate::handlers::user::*;
use crate::handlers::auth::*;
use crate::handlers::upload::*;
use crate::handlers::bucket::*;
use crate::handlers;
use model::*;
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::user::user_list,
    ),
    components(
        schemas(
            UserInfo
        )
    ),
)]
pub struct ApiDoc;

