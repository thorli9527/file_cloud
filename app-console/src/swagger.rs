use crate::handlers::user::*;
use utoipa::OpenApi;
use crate::handlers::auth::*;
// use crate::handlers::upload::*;
// use crate::handlers::bucket::*;
use crate::handlers;
use model::*;
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::user::user_list,
        handlers::user::user_delete,
        handlers::user::user_new,
        handlers::user::user_change_key,
        handlers::user::user_change_password,

        handlers::auth::login,
    ),
    components(schemas(UserInfo,LoginInfo))
)]
pub struct ApiDoc;
