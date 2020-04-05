use std::collections::HashMap;
use uuid::Uuid;
use actix::prelude::*;
use crate::websocket::{WebsocketConnection, WebsockeMessageEvent, WebsocketState};
use crate::messages::{RegisterConnection, PublishMessage, Shutdown};
use crate::redis::RedisActor;

type Endpoint = String;

pub struct AppData {
  clients: HashMap<Endpoint, HashMap<Uuid, Addr<WebsocketConnection>>>,
  redis: Option<RedisActor>
}

impl AppData {
  pub fn new() -> AppData {
    AppData {
      clients: HashMap::new(),
      redis: None,
    }
  }

  pub fn new_with_redis(redis_url: String) -> AppData {
    let mut app_data = AppData::new();
    let redis = RedisActor::new(redis_url);
    app_data.redis = Some(redis);
    app_data
  }

  pub fn insert(&mut self, endpoint: &Endpoint, id: Uuid, addr: Addr<WebsocketConnection>) {
    if let Some(endpoint_sockets) = self.clients.get_mut(endpoint) {
      endpoint_sockets.insert(id, addr);
    } else {
      // insert new socket
      let mut new_endpoint_sockets = HashMap::new();
      new_endpoint_sockets.insert(id, addr);
      self.clients.insert(endpoint.to_string(), new_endpoint_sockets);
    }
  }
}

impl Actor for AppData {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // increase capacity of message backlog
    ctx.set_mailbox_capacity(100);

    if let Some(redis) = self.redis {
      ctx.spawn(redis.connect().and_then(|| {
        
      }));
    }
  }
}

impl Handler<RegisterConnection> for AppData {
  type Result = ();

  fn handle(&mut self, msg: RegisterConnection, _: &mut Context<Self>) {
    info!("Registering connection for endpoint: {}, id: {}", msg.endpoint, msg.id);
    self.insert(&msg.endpoint, msg.id, msg.connection.clone());
    // if let Some(redis_addr) = &self.redis_addr {
    //   redis_addr.do_send(msg);
    // }
  }
}

impl Handler<PublishMessage> for AppData {
  type Result = ();

  fn handle(&mut self, event: PublishMessage, _ctx: &mut Self::Context) {
    if let Some(endpoint_sockets) = self.clients.get(&event.endpoint) {
      for (_, addr) in endpoint_sockets {
        addr.do_send(WebsockeMessageEvent {
          message: event.message.clone(),
        });
      }
    }

    // if let Some(redis_addr) = &self.redis_addr {
    //   redis_addr.do_send(event);
    // }
  }
}

impl Handler<WebsocketState> for AppData {
    type Result = ();

    fn handle(&mut self, msg: WebsocketState, _ctx: &mut Self::Context) {
      match msg {
        WebsocketState::Closed { endpoint, id } => match self.clients.get_mut(&endpoint) {
          Some(endpoint_sockets) => {
            info!("Removing socket with id: {}", id);
            endpoint_sockets.remove(&id);

            if endpoint_sockets.is_empty() {
              self.clients.remove(&endpoint);
            }
          },
          None => {},
        },
      };
    }
}

impl Handler<Shutdown> for AppData {
    type Result = ();

    fn handle(&mut self, _msg: Shutdown, _ctx: &mut Self::Context) {
      debug!("Stopping current system");
      System::current().stop();
    }
}