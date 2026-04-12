use super::CLIENT_TIMEOUT;
use super::HEARTBEAT_INTERVAL;
use crate::config::CONFIG;
use crate::http::rest_api::get_user_pool;
use ::actix::ActorFutureExt;
use ::actix::Message as ActixMessage;
use ::actix::StreamHandler;
use ::actix::{Actor, ActorContext, Addr, AsyncContext, Handler};
use ::actix_identity::Identity;
use ::actix_web::Error;
use ::actix_web::HttpRequest;
use ::actix_web::HttpResponse;
use ::actix_web::error::ErrorUnauthorized;
use ::actix_web::get;
use ::actix_web::web::Data;
use ::actix_web::web::Payload;
use ::actix_web_actors::ws;
use ::actix_web_actors::ws::Message;
use ::actix_web_actors::ws::ProtocolError;
use ::actix_web_actors::ws::WebsocketContext;
use ::aquarius::client::AquariusClient;
use ::aquarius::event::AquariusEvent;
use ::aquarius::messages::Heat as AquariusHeat;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::db::aquarius::Aquarius;
use ::db::aquarius::model::Heat as DbHeat;
use ::db::tiberius::TiberiusPool;
use ::db::tiberius::user_pool::UserPoolManager;
use ::db::timekeeper::TimeStrip;
use ::db::timekeeper::Timestamp;
use ::serde::Deserialize;
use ::serde::Serialize;
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

/// A timekeeping command sent from the client to trigger timekeeping actions on the server.
/// Direction: Client -> Server
#[derive(Debug, Deserialize)]
enum TimekeepingCommand {
    /// Add a start timestamp to the timestrip
    AddStart {
        /// The time of the timestamp to add
        time: Option<DateTime<Utc>>,
    },
    /// Add a finish timestamp to the timestrip
    AddFinish {
        /// The time of the timestamp to add
        time: Option<DateTime<Utc>>,
    },
    /// Delete a timestamp from the timestrip
    DeleteTimestamp {
        /// The time of the timestamp to delete
        time: DateTime<Utc>,
    },
    /// Update a timestamp with a new heat number
    UpdateTimestamp {
        /// The time of the timestamp to update
        time: DateTime<Utc>,
        /// The new heat number to set for the timestamp
        heat_nr: i16,
    },
    /// Get the current timestrip data
    GetTimestrip,
    /// Get the current heats open in Aquarius
    GetHeatsReadyToStart,
}

/// Events sent from the server to the client to update the UI with timekeeping-related information.
/// Direction: Server -> Client
#[derive(ActixMessage)]
#[rtype(result = "()")]
#[derive(Debug, Serialize)]
enum ServerEvent {
    /// Event to send the current heats open in Aquarius to the client
    AquariusHeats { heats: Vec<AquariusHeat> },
    /// Event to send the current timestrip data to the client
    TimeStrip { time_stamps: Vec<Timestamp> },
    /// Event to send a single timestamp update to the client
    Timestamp { timestamp: Timestamp },
    /// Event to send the current heats ready to start to the client
    HeatsReadyToStart { heats: Vec<DbHeat> },
    /// Event to send an error message to the client
    Error {
        /// The error message to send to the client
        error: String,
    },
}

/// Message to trigger persisting a timestamp and sending it back to the client
/// Direction: Server -> Server
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct AddTimestamp {
    /// The time of the timestamp to add (if None, the current time will be used)
    time: Option<DateTime<Utc>>,
    /// The split number for the timestamp (0 for start, 64 for finish)
    split: u8,
}

/// Message to trigger deleting a timestamp
/// Direction: Server -> Server
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct DeleteTimestamp {
    /// The time of the timestamp to delete
    time: DateTime<Utc>,
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
struct UpdateTimestamp {
    /// The time of the timestamp to update
    time: DateTime<Utc>,
    /// The new heat number to set for the timestamp
    heat_nr: i16,
}

/// Message to trigger loading the current timestrip and sending it back to the client
/// Direction: Server -> Server
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct GetTimestrip;

/// Message to trigger loading the current heats ready to start from Aquarius and sending them back to the client
/// Direction: Server -> Server
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct GetHeatsReadyToStart;

struct TimekeepingActor {
    heart_beat: Instant,
    aquarius_client: Arc<AquariusClient>,
    aquarius_db: Data<Aquarius>,
    heats: Arc<RwLock<Vec<AquariusHeat>>>,
    event_receiver: Option<Receiver<AquariusEvent>>,
    time_strip: Arc<::tokio::sync::RwLock<TimeStrip>>,
}

impl TimekeepingActor {
    async fn new(pool: Arc<TiberiusPool>, aquarius_db: Data<Aquarius>) -> Self {
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
            time_strip: Arc::new(::tokio::sync::RwLock::new(TimeStrip::load(pool.clone()).await.unwrap())),
            aquarius_db,
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
}

/// WebSocket handler for timekeeping commands from the client.
/// Direction: Client -> Server
impl StreamHandler<Result<Message, ProtocolError>> for TimekeepingActor {
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
                Ok(cmd_msg) => match cmd_msg {
                    TimekeepingCommand::AddStart { time } => ctx.address().do_send(AddTimestamp { split: 0, time }),
                    TimekeepingCommand::AddFinish { time } => ctx.address().do_send(AddTimestamp { split: 64, time }),
                    TimekeepingCommand::GetTimestrip => ctx.address().do_send(GetTimestrip),
                    TimekeepingCommand::DeleteTimestamp { time } => ctx.address().do_send(DeleteTimestamp { time }),
                    TimekeepingCommand::UpdateTimestamp { time, heat_nr } => {
                        ctx.address().do_send(UpdateTimestamp { time, heat_nr })
                    }
                    TimekeepingCommand::GetHeatsReadyToStart => ctx.address().do_send(GetHeatsReadyToStart),
                },
                Err(err) => {
                    ctx.address().do_send(ServerEvent::Error {
                        error: format!("Failed to parse command: {err}"),
                    });
                }
            },
            Ok(Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl Handler<ServerEvent> for TimekeepingActor {
    type Result = ();

    fn handle(&mut self, event: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        let json = serde_json::to_string(&event).unwrap_or_default();
        ctx.text(json);
    }
}

impl Handler<AddTimestamp> for TimekeepingActor {
    type Result = ();

    fn handle(&mut self, msg: AddTimestamp, ctx: &mut Self::Context) -> Self::Result {
        let time_strip = self.time_strip.clone();
        let split = msg.split;

        ctx.wait(
            actix::fut::wrap_future(async move {
                let mut time_strip = time_strip.write().await;
                let timestamp = match split {
                    0 => time_strip
                        .add_start(msg.time)
                        .await
                        .map_err(|err| format!("Failed to add start timestamp: {err}"))?,
                    64 => time_strip
                        .add_finish(msg.time)
                        .await
                        .map_err(|err| format!("Failed to add finish timestamp: {err}"))?,
                    _ => {
                        return Err(format!("Invalid split number: {split}"));
                    }
                };
                Ok(timestamp)
            })
            .map(
                |result: Result<Timestamp, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    let event = match result {
                        Ok(timestamp) => ServerEvent::Timestamp { timestamp },
                        Err(error) => ServerEvent::Error { error },
                    };
                    ctx.address().do_send(event);
                },
            ),
        );
    }
}

impl Handler<DeleteTimestamp> for TimekeepingActor {
    type Result = ();

    fn handle(&mut self, msg: DeleteTimestamp, ctx: &mut Self::Context) -> Self::Result {
        let time_strip = self.time_strip.clone();

        ctx.wait(
            actix::fut::wrap_future(async move {
                let mut time_strip = time_strip.write().await;
                let timestamp = time_strip
                    .delete(&msg.time)
                    .await
                    .map_err(|err| format!("Failed to delete timestamp: {err}"))?;
                Ok(timestamp)
            })
            .map(
                |result: Result<Timestamp, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    if let Err(error) = result {
                        let event = ServerEvent::Error { error };
                        ctx.address().do_send(event);
                    }
                },
            ),
        );
    }
}

impl Handler<UpdateTimestamp> for TimekeepingActor {
    type Result = ();

    fn handle(&mut self, msg: UpdateTimestamp, ctx: &mut Self::Context) -> Self::Result {
        let time_strip = self.time_strip.clone();
        ctx.wait(
            actix::fut::wrap_future(async move {
                let mut time_strip = time_strip.write().await;
                let timestamp = time_strip.get_by_time(&msg.time).cloned();
                if let Some(timestamp) = timestamp {
                    let timestamp = time_strip
                        .set_heat_nr(&timestamp, msg.heat_nr)
                        .await
                        .map_err(|err| format!("Failed to update timestamp heat number: {err}"))?;
                    Ok(timestamp)
                } else {
                    Err(format!("Timestamp with time {} not found", msg.time))
                }
            })
            .map(
                |result: Result<Timestamp, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    let event = match result {
                        Ok(timestamp) => ServerEvent::Timestamp { timestamp },
                        Err(error) => ServerEvent::Error { error },
                    };
                    ctx.address().do_send(event);
                },
            ),
        );
    }
}

impl Handler<GetTimestrip> for TimekeepingActor {
    type Result = ();

    fn handle(&mut self, _msg: GetTimestrip, ctx: &mut Self::Context) -> Self::Result {
        let time_strip = self.time_strip.clone();

        ctx.wait(
            actix::fut::wrap_future(async move {
                let time_strip = time_strip.read().await;
                Ok(time_strip.to_vec())
            })
            .map(
                |result: Result<Vec<Timestamp>, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    let event = match result {
                        Ok(time_stamps) => ServerEvent::TimeStrip { time_stamps },
                        Err(error) => ServerEvent::Error { error },
                    };
                    ctx.address().do_send(event);
                },
            ),
        );
    }
}

impl Handler<GetHeatsReadyToStart> for TimekeepingActor {
    type Result = ();

    fn handle(&mut self, _msg: GetHeatsReadyToStart, ctx: &mut Self::Context) -> Self::Result {
        let aquarius_db = self.aquarius_db.clone();

        ctx.wait(
            actix::fut::wrap_future(async move {
                aquarius_db
                    .get_heats_ready_to_start()
                    .await
                    .map_err(|err| format!("Failed to read heats ready to start from Aquarius DB: {err}"))
            })
            .map(
                |result: Result<Vec<DbHeat>, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    let event = match result {
                        Ok(heats) => ServerEvent::HeatsReadyToStart { heats },
                        Err(error) => ServerEvent::Error { error },
                    };
                    ctx.address().do_send(event);
                },
            ),
        );
    }
}

impl Actor for TimekeepingActor {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Timekeeping websocket actor started");
        self.start_heart_beat(ctx);

        if let Some(event_receiver) = self.event_receiver.take() {
            let aquarius_client = self.aquarius_client.clone();
            let heats = self.heats.clone();
            let address = ctx.address();
            thread::spawn(move || receive_aquarius_events(event_receiver, aquarius_client, heats, address));
            ctx.address().do_send(GetTimestrip);
        } else {
            error!("Failed to take event receiver for timekeeping websocket actor");
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("Timekeeping websocket actor stopped");
        self.aquarius_client.shutdown();
    }
}

#[get("/timekeeping")]
async fn get_timekeeping_ws(
    request: HttpRequest,
    stream: Payload,
    identity: Option<Identity>,
    aquarius_db: Data<Aquarius>,
    user_pool_manager: Data<UserPoolManager>,
) -> Result<HttpResponse, Error> {
    if let Some(ref identity) = identity {
        let pool = get_user_pool(identity, &user_pool_manager).await?;
        let actor = TimekeepingActor::new(pool, aquarius_db.clone()).await;
        ws::start(actor, &request, stream)
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

fn receive_aquarius_events(
    receiver: Receiver<AquariusEvent>,
    aquarius_client: Arc<AquariusClient>,
    heats: Arc<RwLock<Vec<AquariusHeat>>>,
    addr: Addr<TimekeepingActor>,
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
        addr.do_send(ServerEvent::AquariusHeats {
            heats: heats.read().unwrap().clone(),
        });
    }
}
