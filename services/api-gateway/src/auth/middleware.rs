use std::{future::{ready, Ready}, rc::Rc};

use actix_web::{body::{EitherBody, MessageBody}, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, http::{header::AUTHORIZATION, StatusCode}, web::Data, Error, HttpMessage, HttpResponse};
use futures_util::future::LocalBoxFuture;

use super::jwt::JwtValidator;



#[derive(Clone)]
pub struct JwtConfig {
    pub required_roles: Option<Vec<String>>,
    pub required_scopes: Option<Vec<String>>,
    pub require_admin: bool,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            required_roles: None,
            required_scopes: None,
            require_admin: false,
        }
    }
}

impl JwtConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require_roles(mut self, roles: Vec<&str>) -> Self {
        self.required_roles = Some(roles.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn require_scopes(mut self, scopes: Vec<&str>) -> Self {
        self.required_scopes = Some(scopes.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn require_admin(mut self) -> Self {
        self.require_admin = true;
        self
    }
}

pub struct JwtMiddleware {
    config: JwtConfig,
}

impl JwtMiddleware {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }
}

impl Default for JwtMiddleware {
    fn default() -> Self {
        Self::new(JwtConfig::default())
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
    config: JwtConfig,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let validator = match req.app_data::<Data<JwtValidator>>() {
                Some(v) => v.get_ref().clone(),
                None => {
                    tracing::error!("JwtValidator not found in app data");
                    return Ok(create_error_response(req, "", StatusCode::INTERNAL_SERVER_ERROR));
                }
            };

            let auth_header = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok());

            let auth_header = match auth_header {
                Some(header) => header,
                None => {
                    return Ok(create_error_response(req, "Missing Authorization header", StatusCode::UNAUTHORIZED));
                }
            };

            if !auth_header.starts_with("Bearer ") {
                return Ok(create_error_response(req, "Invalid Authorization header format", StatusCode::UNAUTHORIZED));
            }
        
            let token = auth_header.trim_start_matches("Bearer ");
            
            if token.is_empty() {
                return Ok(create_error_response(req, "Empty token", StatusCode::UNAUTHORIZED));
            }

            let claims = match validator.validate_token(&token).await {
                Ok(claims) => claims,
                Err(e) => {
                    tracing::warn!("JWT validation failed: {}", e);
                    return Ok(create_error_response(req, &format!("Invalid token: {}", e), StatusCode::UNAUTHORIZED));
                }
            };

            if let Some(required_roles) = &config.required_roles {
                let role_refs: Vec<&str> = required_roles.iter().map(|s| s.as_str()).collect();
                if !claims.has_any_role(&role_refs) {
                    tracing::warn!("User {} lacks required roles: {:?}", claims.sub, required_roles);
                    return Ok(create_error_response(req, "Insufficient permissions", StatusCode::FORBIDDEN));
                }
            }

            if config.require_admin && !claims.is_admin() {
                tracing::warn!("User {} is not an admin but admin access required", claims.sub);
                return Ok(create_error_response(req, "Admin access required", StatusCode::FORBIDDEN));
            }

            req.extensions_mut().insert(claims);

            let res = service.call(req).await?;
            Ok(res.map_body(|_, body| EitherBody::left(body)))
        })
    }
}

fn create_error_response<B>(req: ServiceRequest, message: &str, status: StatusCode) -> ServiceResponse<EitherBody<B>>
where
    B: MessageBody
{
    let (req, _) = req.into_parts();
    let response = HttpResponse::build(status)
        .json(serde_json::json!({
            "error": status.canonical_reason().unwrap_or("Unknown error"),
            "message": message
        }))
        .map_body(|_, body| EitherBody::right(body));
    
    ServiceResponse::new(req, response)
}

impl JwtMiddleware {
    pub fn default() -> Self {
        Self::new(JwtConfig::default())
    }

    pub fn admin_only() -> Self {
        Self::new(JwtConfig::new().require_admin())
    }

    pub fn require_roles(roles: Vec<&str>) -> Self {
        Self::new(JwtConfig::new().require_roles(roles))
    }

    pub fn require_scopes(scopes: Vec<&str>) -> Self {
        Self::new(JwtConfig::new().require_scopes(scopes))
    }
}