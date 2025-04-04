
pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

use prost::bytes::Bytes;
use tonic::transport::Body;

/// Auto trait that describes a type, that can be converted to Grpc ServiceClient
/// # Examples
/// ```rust
/// use gigachat::giga::chat_service_client::ChatServiceClient;
/// use gigachat::faces:: GrpcInner;
/// fn make_client<I: GrpcInner>(inner: I) -> ChatServiceClient<I> {
///     ChatServiceClient::new(inner)
/// }
/// ```
pub trait GrpcInner: tonic::client::GrpcService<
    tonic::body::BoxBody, 
    Error: Into<StdError> + Send, 
    ResponseBody: Body<Data = Bytes, Error: Into<StdError> + Send> + Send  + 'static,
    Future: Send
> {}

impl <B, BE, E, S> GrpcInner for S where 
    S: tonic::client::GrpcService<tonic::body::BoxBody, Error = E, ResponseBody = B>, 
    S::Future: Send,
    E: Into<StdError> + Send, 
    B: Body<Data = Bytes, Error = BE> + Send  + 'static,
    BE: Into<StdError> + Send
{}

#[cfg(feature="tower")]
pub use crate::service::ChatService;