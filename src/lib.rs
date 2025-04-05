pub mod giga;
pub mod auth;
pub mod faces;

/// Create [faces::GrpcInner] from auth token and [auth::Scope]
pub async fn make_grpc_service(auth_token: String, scope: auth::Scope) -> Result<impl faces::GrpcInner + Send + Clone + 'static, auth::Error> { 
    let tls = tonic::transport::ClientTlsConfig::new().with_native_roots();
    Ok(tonic::service::interceptor::InterceptedService::new(
        tonic::transport::Channel::from_static("https://gigachat.devices.sberbank.ru")
            .tls_config(tls).unwrap() //never fail, because that static link is correct
            .connect_lazy(), 
        auth::TokenInterceptor::new(auth_token, scope).await?
    ))
}

#[cfg(feature = "tower")]
pub mod service;