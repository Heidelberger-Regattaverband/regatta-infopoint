use super::CLIENT_TIMEOUT;
use super::HEARTBEAT_INTERVAL;
use crate::config::CONFIG;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use crate::http::rest_api::get_user_pool;
use ::actix::StreamHandler;
use ::actix::{Actor, ActorContext, AsyncContext};
use ::actix_identity::Identity;
use ::actix_web::Error;
use ::actix_web::HttpRequest;
use ::actix_web::HttpResponse;
use ::actix_web::Responder;
use ::actix_web::error::ErrorInternalServerError;
use ::actix_web::error::ErrorUnauthorized;
use ::actix_web::get;
use ::actix_web::web::Data;
use ::actix_web::web::Json;
use ::actix_web::web::Payload;
use ::actix_web_actors::ws;
use ::actix_web_actors::ws::Message;
use ::actix_web_actors::ws::ProtocolError;
use ::actix_web_actors::ws::WebsocketContext;
use ::aquarius::client::AquariusClient;
use ::aquarius::event::AquariusEvent;
use ::db::tiberius::user_pool::UserPoolManager;
use ::db::timekeeper::TimeStamp;
use ::db::timekeeper::TimeStrip;
use ::std::sync::mpsc;
use ::std::sync::mpsc::Receiver;
use ::std::thread;
use ::std::time::Instant;
use ::tracing::debug;
use ::tracing::error;
use ::tracing::trace;
use ::tracing::warn;

struct WsTimekeeping {
    heart_beat: Instant,
    aquarius_client: AquariusClient,
}

impl WsTimekeeping {
    fn new() -> Self {
        let (aquarius_event_sender, aquarius_event_receiver) = mpsc::channel();
        let instance = Self {
            heart_beat: Instant::now(),
            aquarius_client: AquariusClient::new(
                &CONFIG.aquarius_host,
                CONFIG.aquarius_port,
                CONFIG.aquarius_timeout,
                aquarius_event_sender,
            )
            .unwrap(),
        };
        thread::spawn(move || receive_aquarius_events(aquarius_event_receiver));
        instance
    }

    fn start_heart_beat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.heart_beat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Timekeeping websocket heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }
            ctx.ping(b"");
        });
    }
}

fn receive_aquarius_events(receiver: Receiver<AquariusEvent>) {
    debug!("Starting AquariusEvent receiver thread for timekeeping websocket");
    while let Ok(event) = receiver.recv() {
        debug!("Received AquariusEvent: {:?}", event);
    }
    debug!("AquariusEvent receiver thread for timekeeping websocket stopped");
}

impl Actor for WsTimekeeping {
    type Context = WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Timekeeping websocket actor started");
        self.start_heart_beat(ctx);
    }

    /// Method is called on actor stop.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("Timekeeping websocket actor stopped");
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WsTimekeeping {
    /// This method is called for every message received from the websocket client
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        trace!(?msg, "Received timekeeping websocket message");
        match msg {
            Ok(Message::Ping(msg)) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(Message::Pong(_)) => {
                self.heart_beat = Instant::now();
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

#[get("/timekeeping")]
async fn get_timekeeping_ws(
    request: HttpRequest,
    stream: Payload,
    identity: Option<Identity>,
) -> Result<HttpResponse, Error> {
    if identity.is_some() {
        ws::start(WsTimekeeping::new(), &request, stream)
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

#[utoipa::path(
    description = "Get the timestrip data for the active regatta. Requires authentication.",
    context_path = PATH,
    responses(
        (status = 200, description = "Timestrip data", body = Vec<TimeStamp>),
        (status = 401, description = "Unauthorized", body = String, example = "Unauthorized"),
        (status = 500, description = INTERNAL_SERVER_ERROR)
    )
)]
#[get("/regattas/active/timestrip")]
async fn get_timestrip(
    identity: Option<Identity>,
    user_pool_manager: Data<UserPoolManager>,
) -> Result<impl Responder, Error> {
    if let Some(identity) = identity
        && let Ok(id) = identity.id()
        && id == "sa"
    {
        let pool = get_user_pool(&identity, &user_pool_manager).await?;
        let mut client = pool.get().await.map_err(|err| {
            error!(%err, "Failed to get DB client from pool");
            ErrorInternalServerError(err)
        })?;
        let timestrip = TimeStrip::load(&mut client).await.map_err(|err| {
            error!(%err, "Failed to load timestrip data");
            ErrorInternalServerError(err)
        })?;
        return Ok(Json(timestrip.time_stamps));
    }
    Err(ErrorUnauthorized("Unauthorized"))
}
