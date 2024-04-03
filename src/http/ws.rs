use actix::{Actor, StreamHandler};
use actix_identity::Identity;
use actix_web::{error::ErrorUnauthorized, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

/// Define HTTP actor
struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload, opt_user: Option<Identity>) -> Result<HttpResponse, Error> {
    if opt_user.is_some() {
        let resp = ws::start(MyWs {}, &req, stream);
        println!("{:?}", resp);
        resp
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

/// Configure the websocket service
pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/ws").route(web::get().to(index)));
}
