use actix::prelude::*;
use uuid::Uuid;
use crate::websocket::{WebsocketConnection};

#[derive(Message, Clone)]
pub struct RegisterConnection {
  pub id: Uuid,
  pub endpoint: String,
  pub connection: Addr<WebsocketConnection>,
}

#[derive(Message, Clone)]
pub struct PublishMessage {
  pub endpoint: String,
  pub message: String,
}

#[derive(Message, Clone)]
pub struct Shutdown;