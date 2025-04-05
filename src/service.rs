use std::pin::Pin;

use crate::faces::{GrpcInner, StdError};
use tower::Service;

use crate::giga::{chat_service_client::ChatServiceClient, ChatRequest, ChatResponse};

/// "Auto" trait that describes a tower's service which consumes ChatRequest and returns ChatResponse.
/// # Examples
/// 
/// ``` rust
/// use gigachat::faces::*; 
/// use gigachat::giga::{ChatRequest, ChatResponse, Message};
/// use tower::Service;
/// 
/// fn make_service(inner: impl GrpcInner + Clone + Send + 'static) -> impl ChatService {
///     let service = gigachat::service::Chat::new(inner);
///     let service = LoggerService(service);
///     // let service = RetryLayer::new(some_policy).layer(service);
///     service
/// }
/// ```
pub trait ChatService: Service<ChatRequest, Response = ChatResponse, Error = StdError, Future: Send> {}
impl <S> ChatService for S where S: Service<ChatRequest, Response = ChatResponse, Error = StdError, Future: Send> {}

#[derive(Clone)]
pub struct Chat<I: Clone>(I);

impl<I: GrpcInner + Send + Clone + 'static> Service<ChatRequest> for Chat<I> {
    type Response = ChatResponse;
    type Error = StdError;
    type Future = Pin<Box< dyn Future<Output = Result<ChatResponse, StdError>> + Send  >>;
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx).map_err(Into::into)
    }
    fn call(&mut self, req: ChatRequest) -> Self::Future {
        let client = ChatServiceClient::new(self.0.clone());
        Box::pin(async move{
            call_chat(client, req).await
        })
    }
}

async fn call_chat(mut client: ChatServiceClient<impl GrpcInner>, req: ChatRequest) -> Result<ChatResponse, StdError> {
    Ok(client.chat(req).await?.into_inner())
}

impl<I: GrpcInner + Send + Clone + 'static> Chat<I> {
    pub fn new(inner: I) -> Self {
        Self(inner)
    }
}
