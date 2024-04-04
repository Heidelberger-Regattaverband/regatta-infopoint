use actix::{Actor, StreamHandler};
use actix_identity::Identity;
use actix_web::{
    error::ErrorUnauthorized,
    get,
    web::{Payload, ServiceConfig},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{start, Message, ProtocolError, WebsocketContext};

/// Define HTTP actor
struct MonitoringWs;

impl Actor for MonitoringWs {
    type Context = WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for MonitoringWs {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Text(text)) => ctx.text(text),
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/ws")]
async fn index(req: HttpRequest, stream: Payload, opt_user: Option<Identity>) -> Result<HttpResponse, Error> {
    if opt_user.is_some() {
        let resp = start(MonitoringWs {}, &req, stream);
        println!("{:?}", resp);
        resp
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

/// Configure the websocket service
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(index);
}
