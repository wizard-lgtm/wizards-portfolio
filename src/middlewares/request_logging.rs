use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::time::Instant;

use crate::logging::{LoggerDb, RequestLogger, request_logger::RequestLog};

pub struct RequestLogging;

impl<S, B> Transform<S, ServiceRequest> for RequestLogging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggingMiddleware { service }))
    }
}

pub struct RequestLoggingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let request_id = RequestLogger::create_request_id();
        
        // Extract request info before moving req
        let ip = RequestLogger::extract_ip(req.request());
        let user_agent = RequestLogger::extract_user_agent(req.request());
        let method = RequestLogger::extract_method(req.request());
        let path = RequestLogger::extract_path(req.request());
        
        // Skip logging for static files and log endpoints themselves
        let should_log = !path.starts_with("/static") && !path.starts_with("/api/log");
        
        // Get logger_db reference
        let logger_db = req.app_data::<web::Data<LoggerDb>>().cloned();
        
        req.extensions_mut().insert(request_id.clone());
        
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            
            if should_log {
                let elapsed = start.elapsed().as_millis() as u64;
                let status_code = res.status().as_u16();
                
                if let Some(logger_db) = logger_db {
                    let request_log = RequestLog {
                        id: None,
                        timestamp: mongodb::bson::DateTime::now(),
                        request_id,
                        ip_address: ip,
                        user_agent,
                        method,
                        path,
                        status_code,
                        response_time_ms: elapsed,
                        location: None,
                        country: None,
                        city: None,
                    };
                    
                    // Log asynchronously without blocking the response
                    let logger_db_clone = logger_db.clone();
                    actix_web::rt::spawn(async move {
                        if let Err(e) = logger_db_clone.log_request(request_log).await {
                            eprintln!("Failed to log request: {}", e);
                        }
                    });
                }
            }
            
            Ok(res)
        })
    }
}
