use crate::handlers::user::*;
use crate::handlers::user_bucket::*;
use utoipa::OpenApi;
use crate::handlers::auth::*;
// use crate::handlers::upload::*;
use crate::handlers::bucket::*;
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

        handlers::bucket::list,
        handlers::bucket::save,
        handlers::bucket::bucket_delete,

        handlers::file::file_list,

        handlers::user_bucket::user_bucket_list,
        handlers::user_bucket::user_bucket_delete

    ),
    components(schemas(UserInfo,LoginInfo,Bucket,BucketInfoResult))
)]
pub struct ApiDoc;
