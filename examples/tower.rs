use gigachat::faces::*; 
use gigachat::giga::{ChatRequest, ChatResponse, Message};
use tower::Service;

fn make_service(inner: impl GrpcInner + Clone + Send + 'static) -> impl ChatService {
    let service = gigachat::service::Chat::new(inner);
    let service = LoggerService(service);
    // let service = RetryLayer::new(some_policy).layer(service);
    service
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, Default::default()).unwrap();
    let inner = gigachat::make_grpc_service(std::env::var("TOKEN")?, gigachat::auth::Scope::Pers).await?;
    let mut service = make_service(inner);
    let messages = vec![Message{ role: "user".into(), content: "Расскажи анекдот.".into(), ..Default::default() }];
    let ChatResponse { alternatives, usage,.. } = service.call(ChatRequest{ 
        model: "GigaChat".into(),
        messages, 
        ..Default::default() 
    }).await.map_err(|e|e as Box<dyn std::error::Error>)?; //к сожалению, приходится явно приводить тип ошибки
    for a in alternatives {
        if let Some(m) = a.message {
            println!("{}", m.content);
        }
    }
    if let Some(usage) = usage {
        println!("Потрачено {} токенов", usage.total_tokens);
    }
    
    Ok(())
}

struct LoggerService<S>(S);

impl<S,R> Service<R> for LoggerService<S> where S: Service<R>, R: std::fmt::Debug {
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        log::info!("req: {request:?}");
        self.0.call(request)
    }
}