use uuid::Uuid;
use std::collections::HashMap;
use actix::prelude::*;
use crate::websocket::{WebsocketConnection, WebsockeMessageEvent, WebsocketState};

#[derive(Message)]
pub struct RegisterConnection {
  pub id: Uuid,
  pub endpoint: String,
  pub connection: Addr<WebsocketConnection>,
}

#[derive(Message)]
pub struct PublishMessage {
  pub endpoint: String,
  pub message: String,
}

#[derive(Message)]
pub struct Shutdown;

type Endpoint = String;

pub struct AppData {
  pub clients: HashMap<Endpoint, HashMap<Uuid, Addr<WebsocketConnection>>>,
}

impl AppData {
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
  }
}

impl Handler<RegisterConnection> for AppData {
  type Result = ();

  fn handle(&mut self, msg: RegisterConnection, _: &mut Context<Self>) {
    info!("Registering connection for endpoint: {}, id: {}", msg.endpoint, msg.id);
    self.insert(&msg.endpoint, msg.id, msg.connection);
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