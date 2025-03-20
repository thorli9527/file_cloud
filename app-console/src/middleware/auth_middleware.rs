use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    web,
};
use common::{result_error_msg, AppState};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::{future::Future, pin::Pin, rc::Rc};

/// Authentication Middleware
pub struct AuthMiddleware {
    pub state: web::Data<AppState>, // Shared application state
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            state: self.state.clone(),
        })
    }
}

/// Middleware Service Struct
pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    state: web::Data<AppState>, // Store the shared state
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service); // ✅ Correct way to clone the service
        let state = self.state.clone();
        let url = req.uri();
        match url {
            _ if url.path().starts_with("/swagger-ui") => {
                Box::pin(async move {
                    let res = srv.call(req).await?;
                    let res = res.map_body(|_, body| EitherBody::new(body));
                    return Ok(res);
                })
            }
            _ if url.path().starts_with("/auth") => {
                Box::pin(async move {
                    let res = srv.call(req).await?;
                    let res = res.map_body(|_, body| EitherBody::new(body));
                    return Ok(res);
                })
            }
            _ if url.path().starts_with("/api-doc/openapi") => {
                Box::pin(async move {
                    let res = srv.call(req).await?;
                    let res = res.map_body(|_, body| EitherBody::new(body));
                    return Ok(res);
                })
            }
            _ => {
                Box::pin(async move {
                    let auth_header = req.headers().get("Authorization");
                    if let Some(auth_value) = auth_header {
                        if let Ok(auth_str) = auth_value.to_str() {
                            if auth_str.starts_with("Session ") {
                                let token_key = &auth_str[8..];
                                let token_option = state.session_cache.get(token_key).await;
                                if let Some(token_value)=token_option {
                                    let res = srv.call(req).await?;
                                    let res = res.map_body(|_, body| EitherBody::new(body));
                                    return Ok(res);
                                }
                                else{
                                    return Ok(req.into_response(
                                        HttpResponse::Unauthorized()
                                            .json(result_error_msg("Unauthorized"))
                                            .map_into_right_body(),
                                    ));
                                }
                            }
                        }
                    }
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(result_error_msg("Unauthorized"))
                            .map_into_right_body(),
                    ));
                })
            }
        }

    }
}
