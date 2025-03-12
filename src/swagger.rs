use crate::handlers;
use handlers::*;
pub use crate::resp::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::common::status,
        handlers::upload::upload,
    ),
    components(
        schemas(
            // MnemonicReq,
            //
        UploadForm,
        )
    ),
    tags((name = "file", description = "文件上传 API"))
)]
pub struct ApiDoc;
