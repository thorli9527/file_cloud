
use utoipa::OpenApi;
// use crate::handlers::user_list;

#[derive(OpenApi)]
#[openapi(
    paths(
        // handlers::user_list,
    ),
    components(
        schemas(

        )
    ),
)]
pub struct ApiDoc;

