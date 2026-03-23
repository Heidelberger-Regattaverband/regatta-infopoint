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
use ::aquarius::messages::Heat;
use ::db::tiberius::TiberiusPool;
use ::db::tiberius::user_pool::UserPoolManager;
use ::db::timekeeper::TimeStamp;
use ::db::timekeeper::TimeStrip;
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
    AddStart,
    /// Add a finish timestamp to the timestrip
    AddFinish,
    /// Get the current timestrip data
    GetTimestrip,
}

/// Events sent from the server to the client to update the UI with timekeeping-related information.
/// Direction: Server -> Client
#[derive(ActixMessage)]
#[rtype(result = "()")]
#[derive(Debug, Serialize)]
enum ServerEvent {
    /// Event to send the current heats open in Aquarius to the client
    AquariusHeats { heats: Vec<Heat> },
    /// Event to send the current timestrip data to the client
    Timestrip { time_stamps: Vec<TimeStamp> },
    /// Event to send a single timestamp update to the client
    Timestamp { time_stamp: TimeStamp },
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
    /// The split number for the timestamp (0 for start, 64 for finish)
    split: u8,
}

/// Message to trigger loading the current timestrip and sending it back to the client
/// Direction: Server -> Server
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct GetTimestrip;

struct TimekeepingActor {
    heart_beat: Instant,
    aquarius_client: Arc<AquariusClient>,
    heats: Arc<RwLock<Vec<Heat>>>,
    event_receiver: Option<Receiver<AquariusEvent>>,
    time_strip: Arc<::tokio::sync::RwLock<TimeStrip>>,
}

impl TimekeepingActor {
    async fn new(pool: Arc<TiberiusPool>) -> Self {
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
                    TimekeepingCommand::AddStart => ctx.address().do_send(AddTimestamp { split: 0 }),
                    TimekeepingCommand::AddFinish => ctx.address().do_send(AddTimestamp { split: 64 }),
                    TimekeepingCommand::GetTimestrip => ctx.address().do_send(GetTimestrip),
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
                match split {
                    0 => time_strip
                        .add_start()
                        .await
                        .map_err(|err| format!("Failed to add start timestamp: {err}"))?,
                    64 => time_strip
                        .add_finish()
                        .await
                        .map_err(|err| format!("Failed to add finish timestamp: {err}"))?,
                    _ => {
                        return Err(format!("Invalid split number: {split}"));
                    }
                }
                time_strip
                    .time_stamps
                    .iter()
                    .next()
                    .cloned()
                    .ok_or("No timestamps available".to_string())
            })
            .map(
                |result: Result<TimeStamp, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    let event = match result {
                        Ok(time_stamp) => ServerEvent::Timestamp { time_stamp },
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

        // let time_stamps = time_strip.time_stamps.clone();
        // ctx.address().do_send(ServerEvent::Timestrip {
        //     time_stamps: time_stamps.into(),
        // });

        ctx.wait(
            actix::fut::wrap_future(async move {
                let time_strip = time_strip.read().await;
                let time_stamps = time_strip.time_stamps.clone();
                Ok(time_stamps.into())
            })
            .map(
                |result: Result<Vec<TimeStamp>, String>, _actor, ctx: &mut WebsocketContext<TimekeepingActor>| {
                    let event = match result {
                        Ok(time_stamps) => ServerEvent::Timestrip { time_stamps },
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
    user_pool_manager: Data<UserPoolManager>,
) -> Result<HttpResponse, Error> {
    if let Some(ref identity) = identity {
        let pool = get_user_pool(identity, &user_pool_manager).await?;
        let actor = TimekeepingActor::new(pool).await;
        ws::start(actor, &request, stream)
    } else {
        Err(ErrorUnauthorized("Unauthorized"))
    }
}

fn receive_aquarius_events(
    receiver: Receiver<AquariusEvent>,
    aquarius_client: Arc<AquariusClient>,
    heats: Arc<RwLock<Vec<Heat>>>,
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
