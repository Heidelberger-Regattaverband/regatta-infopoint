use super::CLIENT_TIMEOUT;
use super::HEARTBEAT_INTERVAL;
use crate::http::monitoring::Monitoring;
use ::actix::Actor;
use ::actix::ActorContext;
use ::actix::AsyncContext;
use ::actix::Handler;
use ::actix::Message as ActixMessage;
use ::actix::StreamHandler;
use ::actix_identity::Identity;
use ::actix_web::web::{Data, Payload};
use ::actix_web::{Error, HttpRequest, HttpResponse, error::ErrorUnauthorized, get};
use ::actix_web_actors::ws::{Message, ProtocolError, WebsocketContext, start};
use ::db::aquarius::Aquarius;
use ::db::tiberius::TiberiusPool;
use ::serde::Serialize;
use ::std::time::Instant;
use ::tracing::trace;
use ::tracing::warn;

/// Actor for monitoring.
struct MonitoringActor {
    /// Timestamp of the last heartbeat received from the client. Used to detect if the client is still alive.
    heart_beat: Instant,
    /// Reference to the Aquarius database. Used to get the monitoring data.
    aquarius_db: Data<Aquarius>,
}

impl MonitoringActor {
    fn new(aquarius: Data<Aquarius>) -> Self {
        Self {
            heart_beat: Instant::now(),
            aquarius_db: aquarius,
        }
    }

    fn start_heart_beat(&self, ctx: &mut <Self as Actor>::Context) {
        let aquarius_db = self.aquarius_db.clone();

        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.heart_beat) > CLIENT_TIMEOUT {
                warn!("Monitoring websocket heartbeat failed, disconnecting!");
                ctx.stop();
            } else {
                Self::send_monitoring(ctx, &aquarius_db);
                ctx.ping(b"");
            }
        });
    }

    fn send_monitoring(ctx: &mut <Self as Actor>::Context, aquarius_db: &Aquarius) {
        let monitoring = Monitoring::new(TiberiusPool::instance(), &aquarius_db.get_cache_stats());
        ctx.address().do_send(MonitoringEvent::Update { monitoring });
    }
}

impl Actor for MonitoringActor {
    type Context = WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Monitoring websocket actor started");
        Self::send_monitoring(ctx, &self.aquarius_db);
        self.start_heart_beat(ctx);
    }

    /// Method is called on actor stop.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("Monitoring websocket actor stopped");
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for MonitoringActor {
    /// This method is called for every message received from the websocket client
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        trace!(?msg, "Received Monitoring websocket message");
        match msg {
            Ok(Message::Ping(msg)) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(Message::Pong(_)) => {
                self.heart_beat = Instant::now();
            }
            Ok(Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
#[derive(Serialize)]
enum MonitoringEvent {
    /// Event to update the monitoring data on the client.
    Update { monitoring: Monitoring },
}

impl Handler<MonitoringEvent> for MonitoringActor {
    type Result = ();

    fn handle(&mut self, event: MonitoringEvent, ctx: &mut Self::Context) -> Self::Result {
        let json = serde_json::to_string(&event).unwrap_or_default();
        ctx.text(json);
    }
}

#[get("/monitoring")]
async fn index(
    request: HttpRequest,
    stream: Payload,
    aquarius: Data<Aquarius>,
    identity: Option<Identity>,
) -> Result<HttpResponse, Error> {
    if identity.is_some() {
        let monitoring_actor = MonitoringActor::new(aquarius);
        start(monitoring_actor, &request, stream)
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}
