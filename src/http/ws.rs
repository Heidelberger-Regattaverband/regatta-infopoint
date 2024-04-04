use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{
    get,
    web::{Payload, ServiceConfig},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{start, Message, ProtocolError, WebsocketContext};
use log::{debug, warn};
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Define HTTP actor
struct WsMonitoring {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl WsMonitoring {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.text("Hello World!");
            ctx.ping(b"");
        });
    }
}

impl Actor for WsMonitoring {
    type Context = WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Websocket actor started");
        self.hb(ctx);
    }

    /// Method is called on actor stop.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Websocket actor stopped");
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for WsMonitoring {
    /// This method is called for every message received from the websocket client
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        debug!("WS: {msg:?}");
        match msg {
            Ok(Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(Message::Text(text)) => ctx.text(text),
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            Ok(Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

#[get("/ws")]
async fn index(request: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    let response = start(WsMonitoring::new(), &request, stream);
    if response.is_ok() {
        debug!("Websocket connection established");
    } else {
        debug!("Websocket connection failed");
    }
    debug!("{:?}", response);
    response
}

/// Configure the websocket service
pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.service(index);
}
