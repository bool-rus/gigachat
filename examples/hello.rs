use std::error::Error;

use gigachat::giga::{chat_service_client::ChatServiceClient, *};
use gigachat::auth::{Scope, TokenInterceptor};
use tonic::transport::{Channel, ClientTlsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tls = ClientTlsConfig::new().with_native_roots();
    let channel = Channel::from_static("https://gigachat.devices.sberbank.ru").tls_config(tls)?.connect_lazy();
    let token = std::env::var("TOKEN")?;
    let mut client = ChatServiceClient::with_interceptor(channel, TokenInterceptor::new(token, Scope::Pers).await?);
    let messages = vec![Message{ role: "user".into(), content: "Привет! Расскажи анекдот.".into(), ..Default::default() }];
    let ChatResponse { alternatives, .. } = client.chat(ChatRequest{ 
        model: "GigaChat".into(),
        messages, 
        ..Default::default() 
    }).await?.into_inner();
    for a in alternatives {
        if let Some(m) = a.message {
            println!("{}", m.content);
        }
    }
    Ok(())
}