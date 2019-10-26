use std::time::{Duration, Instant};
use actix::prelude::*;
use uuid::Uuid;
use actix_web_actors::ws;
use crate::app_data::AppData;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Message)]
pub struct WebsockeMessageEvent {
  pub message: String,
}

#[derive(Message)]
pub enum WebsocketState {
  Closed {
    endpoint: String,
    id: Uuid,
  },
}

pub struct WebsocketConnection {
  pub id: Uuid,
  pub endpoint: String,
  // TODO: Investigate way that Websocket is not coupled with AppData Actor
  state_handler: Addr<AppData>,
  heartbeat: Instant,
}

impl WebsocketConnection {
  pub fn new(endpoint: String, state_handler: Addr<AppData>) -> Self 
  {
    WebsocketConnection {
      id: Uuid::new_v4(),
      endpoint,
      state_handler,
      heartbeat: Instant::now(),
    }
  }

  fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
      // check client heartbeats
      if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
        // heartbeat timed out
        info!("Websocket Client heartbeat failed, disconnecting!");

        // stop actor
        ctx.stop();
        act.state_handler.do_send(WebsocketState::Closed{
          endpoint: act.endpoint.clone(),
          id: act.id.clone(),
        });

        // don't try to send a ping
        return;
      }

      ctx.ping("");
    });
  }
}

impl Actor for WebsocketConnection {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.heartbeat(ctx);
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketConnection {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    debug!("WS: {:?}", msg);
    match msg {
      ws::Message::Ping(msg) => {
        self.heartbeat = Instant::now();
        ctx.pong(&msg);
      }
      ws::Message::Pong(_) => {
        self.heartbeat = Instant::now();
      }
      ws::Message::Text(text) => ctx.text(text),
      ws::Message::Binary(bin) => ctx.binary(bin),
      ws::Message::Close(_) => {
        ctx.stop();
        self.state_handler.do_send(WebsocketState::Closed{
          endpoint: self.endpoint.clone(),
          id: self.id.clone(),
        });
      }
      ws::Message::Nop => (),
    }
  }
}

impl Handler<WebsockeMessageEvent> for WebsocketConnection {
  type Result = ();

  fn handle(&mut self, event: WebsockeMessageEvent, ctx: &mut Self::Context) {
    ctx.text(event.message);
  }
}