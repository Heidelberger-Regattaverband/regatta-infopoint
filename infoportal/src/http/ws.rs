use crate::http::monitoring::Monitoring;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_identity::Identity;
use actix_web::{
    Error, HttpRequest, HttpResponse,
    error::ErrorUnauthorized,
    get,
    web::{Data, Payload},
};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext, start};
use aquarius::tiberius::TiberiusPool;
use log::{debug, warn};
use prometheus::Registry;
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

/// Define HTTP actor
struct WsMonitoring {
    /// Client must send ping at least once per 5 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    registry: Data<Registry>,
}

impl WsMonitoring {
    fn new(registry: Data<Registry>) -> Self {
        Self {
            hb: Instant::now(),
            registry,
        }
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        let registry = self.registry.clone();

        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            Self::send_monitoring(ctx, &registry);
            ctx.ping(b"");
        });
    }

    fn send_monitoring(ctx: &mut WebsocketContext<WsMonitoring>, registry: &Registry) {
        let monitoring = Monitoring::new(TiberiusPool::instance(), registry);
        let json = serde_json::to_string(&monitoring).unwrap();
        ctx.text(json);
    }
}

impl Actor for WsMonitoring {
    type Context = WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Websocket actor started");
        Self::send_monitoring(ctx, &self.registry);
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

#[get("/monitoring")]
async fn index(
    request: HttpRequest,
    stream: Payload,
    registry: Data<Registry>,
    opt_user: Option<Identity>,
) -> Result<HttpResponse, Error> {
    if opt_user.is_some() {
        let response = start(WsMonitoring::new(registry), &request, stream);
        debug!("{:?}", response);
        response
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}
