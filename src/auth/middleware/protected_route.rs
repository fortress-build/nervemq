//! Protected route middleware for role-based access control.
//!
//! Provides middleware to restrict route access based on user authentication
//! and role requirements (admin or regular user).

use std::future::{Future, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_identity::IdentityExt;
use actix_web::dev::{Service, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};

use crate::api::auth::Role;

/// Configuration for protected route access.
///
/// Controls whether a route requires admin privileges or just authentication.
#[derive(Clone)]
pub struct Protected {
    admin_only: bool,
}

impl Protected {
    /// Creates new protection config with specified admin requirement.
    pub fn new(admin_only: bool) -> Self {
        Self { admin_only }
    }

    /// Shorthand to create admin-only route protection.
    pub fn admin_only() -> Self {
        Self::new(true)
    }

    /// Shorthand to create protection requiring only authentication.
    pub fn authenticated() -> Self {
        Self::new(false)
    }
}

impl Default for Protected {
    fn default() -> Self {
        Self::authenticated()
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for Protected
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ProtectedRouteMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> <Self as Transform<S, ServiceRequest>>::Future {
        std::future::ready(Ok(ProtectedRouteMiddleware {
            service: Rc::new(service),
            config: self.clone(),
        }))
    }
}

/// Middleware that enforces route protection rules.
///
/// Validates user identity and role requirements before allowing access.
pub struct ProtectedRouteMiddleware<S> {
    service: Rc<S>,
    config: Protected,
}

impl<S, B> Service<ServiceRequest> for ProtectedRouteMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut Context,
    ) -> Poll<Result<(), <Self as Service<ServiceRequest>>::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> <Self as Service<ServiceRequest>>::Future {
        let svc = Rc::clone(&self.service);

        let api = req
            .app_data::<actix_web::web::Data<crate::service::Service>>()
            .expect("service should be available - this is a bug")
            .clone();

        let required_role = if self.config.admin_only {
            Role::Admin
        } else {
            Role::User
        };

        Box::pin(async move {
            let identity = req.get_identity().map_err(ErrorUnauthorized)?;

            match api.check_user_role(identity, required_role).await {
                Ok(_) => svc.call(req).await,
                Err(e) => Err(ErrorUnauthorized(e)),
            }
        })
    }
}
