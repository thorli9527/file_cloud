use actix_web::web::Data;
use actix_web::{Responder, post, web};
use common::{
    AppError, AppState, PageInfo, build_id, build_md5, result, result_list, result_page,
    result_warn_msg,
};
use model::{Repository, UserInfo, UserRepository};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use utoipa::ToSchema;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(user_list);
    cfg.service(user_delete);
    cfg.service(user_new);
    cfg.service(user_change_key);
    cfg.service(user_change_password);
}
#[utoipa::path(
    post,
    path = "/user/list",
    params(
       // ("hash" = String, description = "The hash of the transaction to query")
    ),
    responses(
        (status = 200, description = "successfully",body = UserInfo)
    )
)]
#[post("/user/list")]
async fn user_list(
    page: web::Json<PageInfo>,
    user_reg: web::Data<UserRepository>,
) -> Result<impl Responder, AppError> {
    let params: HashMap<&str, String> = HashMap::new();
    let page_result = user_reg.dao.query_by_page(params, &page).await?;
    Ok(web::Json(result_page(page_result)))
}

#[utoipa::path(
    post,
    path = "/user/delete/{id}",
    params(
        // ("hash" = String, description = "The hash of the transaction to query")
    ),
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/user/delete/{id}")]
async fn user_delete(
    id: web::Path<String>,
    user_reg: Data<UserRepository>,
) -> Result<impl Responder, AppError> {
    let id_p = format!("{}", id);
    user_reg.dao.del_by_id(id_p).await?;
    Ok(web::Json(result()))
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserNewDto {
    pub user_name: Option<String>,
    pub password: Option<String>,
}

#[utoipa::path(
    post,
    path = "/user/new",
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/user/new")]
async fn user_new(
    user_rep: Data<UserRepository>,
    user: web::Json<UserNewDto>,
) -> Result<impl Responder, AppError> {
    let mut params: HashMap<&str, String> = HashMap::new();
    match &user.user_name {
        Some(user_name) => {
            params.insert("user_name", user_name.to_string());
        }
        None => {
            return Err(AppError::BizError("user_name.is.null".to_string()));
        }
    }
    match &user.password {
        Some(password) => {
            params.insert("password", build_md5(password).to_string());
        }
        None => {
            return Err(AppError::BizError("password.is.null".to_string()));
        }
    }
    params.insert("access_key", build_id());
    params.insert("secret_key", build_id());
    params.insert("id", build_id());
    match user_rep.dao.insert(params).await {
        Ok(_) => {
            return Ok(web::Json(result()));
        }
        Err(e) => match AppError::from(e) {
            AppError::DBError(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                let user_name = match &user.user_name {
                    Some(user_name) => user_name,
                    _ => "",
                };
                let message = format!("用户已存在{}", user_name);
                return Ok(web::Json(result_warn_msg(message.as_str())));
            }
            error => return Err(error),
        },
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
pub struct UserChangeKey {
    pub user_name: String,
}
#[utoipa::path(
    post,
    path = "/user/change/key",
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/user/change/key")]
async fn user_change_key(
    user_rep: web::Data<UserRepository>,
    user: web::Json<UserChangeKey>,
) -> Result<impl Responder, AppError> {
    match user_rep.find_by_name((&*user.user_name).to_string()).await {
        Ok(info) => {
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("access_key", build_id());
            params.insert("secret_key", build_id());
            user_rep.dao.change(&info.id.to_string(), params).await?;
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(web::Json(result()))
}
#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
pub struct UserChangePass {
    pub user_name: String,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
}
#[utoipa::path(
    post,
    path = "/user/change/password",
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/user/change/password")]
async fn user_change_password(
    user_rep: web::Data<UserRepository>,
    user: web::Json<UserChangePass>,
) -> Result<impl Responder, AppError> {
    match user_rep.find_by_name(user.user_name.to_string()).await {
        Ok(info) => {
            let old_password = match &user.old_password {
                Some(old_password) => old_password,
                _ => return Err(AppError::BizError("old_password.is.null".to_string())),
            };
            if build_md5(old_password) != info.password {
                return Err(AppError::BizError(
                    "old_password.is.not.correct".to_string(),
                ));
            }
            let new_password = match &user.new_password {
                Some(new_password) => build_md5(new_password),
                _ => return Err(AppError::BizError("new_password.is.null".to_string())),
            };
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("password", new_password);
            user_rep.dao.change(&info.id.to_string(), params).await?;
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(web::Json(result()))
}
#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
pub struct UserUpPass {
    pub user_name: String,
    pub new_password: Option<String>,
}

#[post("/user/up/password")]
async fn user_up_password(
    user_rep: web::Data<UserRepository>,
    user: web::Json<UserUpPass>,
) -> Result<impl Responder, AppError> {
    match user_rep.find_by_name(user.user_name.to_string()).await {
        Ok(info) => {
            let new_password = match &user.new_password {
                Some(new_password) => build_md5(new_password),
                _ => return Err(AppError::BizError("new_password.is.null".to_string())),
            };
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("password", new_password);
            user_rep.dao.change(&info.id.to_string(), params).await?;
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(web::Json(result()))
}
