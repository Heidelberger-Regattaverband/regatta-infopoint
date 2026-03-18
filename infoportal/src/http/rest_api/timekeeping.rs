use super::CLIENT_TIMEOUT;
use super::HEARTBEAT_INTERVAL;
use crate::config::CONFIG;
use crate::http::rest_api::INTERNAL_SERVER_ERROR;
use crate::http::rest_api::PATH;
use crate::http::rest_api::get_user_pool;
use ::actix::ActorFutureExt;
use ::actix::Message as ActixMessage;
use ::actix::StreamHandler;
use ::actix::{Actor, ActorContext, Addr, AsyncContext, Handler};
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
use ::aquarius::messages::Heat;
use ::db::tiberius::TiberiusPool;
use ::db::tiberius::user_pool::UserPoolManager;
use ::db::timekeeper::TimeStamp;
use ::db::timekeeper::TimeStrip;
use ::serde::Deserialize;
use ::std::sync::Arc;
use ::std::sync::RwLock;
use ::std::sync::mpsc;
use ::std::sync::mpsc::Receiver;
use ::std::thread;
use ::std::time::Instant;
use ::tracing::debug;
use ::tracing::error;
use ::tracing::trace;
use ::tracing::warn;

#[derive(Debug, Deserialize)]
struct TimekeepingCommand {
    command: TimekeepingCommandType,
}

#[derive(Debug, Deserialize)]
enum TimekeepingCommandType {
    AddStart,
    AddFinish,
}

/// Message to trigger sending heats to the WebSocket client
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct SendHeatsMessage;

/// Message to trigger persisting a timestamp and sending it back to the client
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct AddTimestampMessage {
    command: TimekeepingCommandType,
}

struct WsTimekeeping {
    heart_beat: Instant,
    aquarius_client: Arc<AquariusClient>,
    heats: Arc<RwLock<Vec<Heat>>>,
    event_receiver: Option<Receiver<AquariusEvent>>,
    pool: Arc<TiberiusPool>,
}

impl WsTimekeeping {
    fn new(pool: Arc<TiberiusPool>) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        Self {
            heart_beat: Instant::now(),
            aquarius_client: Arc::new(
                AquariusClient::new(
                    &CONFIG.aquarius_host,
                    CONFIG.aquarius_port,
                    CONFIG.aquarius_timeout,
                    event_sender,
                )
                .unwrap(),
            ),
            heats: Arc::new(RwLock::new(Vec::new())),
            event_receiver: Some(event_receiver),
            pool,
        }
    }

    fn start_heart_beat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            if Instant::now().duration_since(act.heart_beat) > CLIENT_TIMEOUT {
                warn!("Timekeeping websocket heartbeat failed, disconnecting!");
                ctx.stop();
            } else {
                ctx.ping(b"");
            }
        });
    }

    fn send_heats(&self, ctx: &mut <Self as Actor>::Context) {
        let heats = self.heats.read().unwrap();
        let json = serde_json::to_string(&*heats).unwrap();
        debug!("Sending heats to timekeeping websocket client: {}", json);
        ctx.text(json);
    }
}

fn receive_aquarius_events(
    receiver: Receiver<AquariusEvent>,
    aquarius_client: Arc<AquariusClient>,
    heats: Arc<RwLock<Vec<Heat>>>,
    addr: Addr<WsTimekeeping>,
) {
    while let Ok(event) = receiver.recv() {
        match event {
            AquariusEvent::HeatListChanged(event) => {
                debug!("Received HeatListChanged event = {:?}", &event);
                if event.opened {
                    let mut heats_lock = heats.write().unwrap();
                    heats_lock.push(event.heat);
                } else {
                    let mut heats_lock = heats.write().unwrap();
                    heats_lock.retain(|heat| heat.id != event.heat.id);
                }
            }
            AquariusEvent::Client(connected) => {
                if connected {
                    if let Ok(open_heats) = aquarius_client.read_open_heats() {
                        let mut heats_lock = heats.write().unwrap();
                        heats_lock.clear();
                        heats_lock.extend(open_heats);
                    } else {
                        error!("Failed to read open heats from Aquarius client");
                    }
                } else {
                    heats.write().unwrap().clear();
                }
            }
        }
        addr.do_send(SendHeatsMessage);
    }
}

impl Handler<SendHeatsMessage> for WsTimekeeping {
    type Result = ();

    fn handle(&mut self, _msg: SendHeatsMessage, ctx: &mut Self::Context) -> Self::Result {
        self.send_heats(ctx);
    }
}

impl Handler<AddTimestampMessage> for WsTimekeeping {
    type Result = ();

    fn handle(&mut self, msg: AddTimestampMessage, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let command = msg.command;

        ctx.wait(
            actix::fut::wrap_future(async move {
                let mut client = pool.get().await.map_err(|err| {
                    error!(%err, "Failed to get DB client from pool");
                    format!("Failed to get DB client: {err}")
                })?;
                let mut time_strip = TimeStrip::load(&mut client).await.map_err(|err| {
                    error!(%err, "Failed to load timestrip");
                    format!("Failed to load timestrip: {err}")
                })?;

                match command {
                    TimekeepingCommandType::AddStart => time_strip.add_start(&mut client).await.map_err(|err| {
                        error!(%err, "Failed to add start timestamp");
                        format!("Failed to add start timestamp: {err}")
                    })?,
                    TimekeepingCommandType::AddFinish => time_strip.add_finish(&mut client).await.map_err(|err| {
                        error!(%err, "Failed to add finish timestamp");
                        format!("Failed to add finish timestamp: {err}")
                    })?,
                }

                Ok::<Vec<TimeStamp>, String>(time_strip.time_stamps)
            })
            .map(
                |result: Result<Vec<TimeStamp>, String>, _actor, ctx: &mut WebsocketContext<WsTimekeeping>| match result
                {
                    Ok(time_stamps) => {
                        let json = serde_json::to_string(&time_stamps).unwrap_or_default();
                        debug!("Sending updated timestrip to client: {}", json);
                        ctx.text(json);
                    }
                    Err(err) => {
                        let error_json = serde_json::json!({"error": err});
                        error!("Failed to persist timestamp: {}", err);
                        ctx.text(error_json.to_string());
                    }
                },
            ),
        );
    }
}

impl Actor for WsTimekeeping {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Timekeeping websocket actor started");
        self.start_heart_beat(ctx);

        if let Some(receiver) = self.event_receiver.take() {
            let aquarius_client = self.aquarius_client.clone();
            let heats = self.heats.clone();
            let addr = ctx.address();
            thread::spawn(move || receive_aquarius_events(receiver, aquarius_client, heats, addr));
        } else {
            error!("Failed to take event receiver for timekeeping websocket actor");
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Timekeeping websocket actor stopped");
        self.aquarius_client.shutdown();
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WsTimekeeping {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        trace!(?msg, "Received timekeeping websocket message");
        match msg {
            Ok(Message::Ping(msg)) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(Message::Pong(_)) => {
                self.heart_beat = Instant::now();
            }
            Ok(Message::Text(text)) => match serde_json::from_str::<TimekeepingCommand>(&text) {
                Ok(command) => {
                    debug!(?command, "Received timekeeping command");
                    ctx.address().do_send(AddTimestampMessage {
                        command: command.command,
                    });
                }
                Err(err) => {
                    warn!(%err, %text, "Failed to parse timekeeping command");
                    let error_json = serde_json::json!({"error": format!("Invalid command: {err}")});
                    ctx.text(error_json.to_string());
                }
            },
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
    user_pool_manager: Data<UserPoolManager>,
) -> Result<HttpResponse, Error> {
    if let Some(ref identity) = identity {
        let pool = get_user_pool(identity, &user_pool_manager).await?;
        ws::start(WsTimekeeping::new(pool), &request, stream)
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
