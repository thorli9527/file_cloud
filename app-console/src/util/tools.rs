use actix_web::{HttpRequest, web};
use common::{AppError, AppState, UserCache};

pub async fn get_session_user(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<UserCache, AppError> {
    let mut token_value = "";
    let option = req.headers().get("Authorization-Token");
    match option {
        Some(auth_value) => {
            if let Ok(token_str) = auth_value.to_str() {
                let option = state.session_cache.get(token_value).await;
                if let Some(user_cache) = option {
                    return Ok(user_cache.clone());
                }
            }
        }
        None => {
            return Err(AppError::NoRight("token.is.null".to_owned()));
        }
    }

    return Err(AppError::NoRight("token.is.null".to_owned()));
}
