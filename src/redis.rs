use url::Url;
use redis::{Client as RedisClient, RedisResult};

// use fred::prelude::*;
// use fred::RedisClient;
// use fred::owned::RedisClientOwned;

// use fred::types::*;
// use fred::error::*;
// use tokio_core::reactor::Core;
// use futures::Future;
// use futures::stream::{
//   self,
//   Stream
// };
// use futures::future::{
//   self,
//   Either
// };

// use std::time::Duration;
use actix::prelude::*;
use crate::messages::{RegisterConnection, PublishMessage};

// fn create_fredis_client(redis_url: String) -> RedisClient {
//   let parsed_redis_url = Url::parse(&redis_url).unwrap();
//   let redis_host = parsed_redis_url.host_str().expect("Invalid redis host");
//   let redis_port = parsed_redis_url.port().expect("Invalid redis port");
//   let redis_config = RedisConfig::new_centralized(
//     redis_host.to_owned(),
//     redis_port,
//     None,
//   );

//   RedisClient::new(redis_config, None)
// }

fn create_redis_client(redis_url: String) -> RedisResult<RedisClient> {
  let client = RedisClient::open(redis_url.as_ref())?;
  // let mut con = client.get_connection()?;
  Ok(client)
}

pub struct RedisActor {
  client: RedisClient,
}

impl Actor for RedisActor {
  type Context = Context<Self>;
}

impl RedisActor {
  pub fn new(redis_url: String) -> RedisActor {
    let client = create_redis_client(redis_url).expect("Redis client failed initialization");
    RedisActor {
      client,
    }
  }

  pub fn connect(&mut self) -> impl ActorFuture<> {
    let mut connection = self.client.get_async_connection();
    // connection.and_then(|con| {

    // })

    actix::fut::wrap_future::<_, Self>(connection)
    // let mut pubsub = connection.as_pubsub();
  }

  pub fn subscribe(&mut self, channel: String) {
    
  }
}

impl Handler<RegisterConnection> for RedisActor {
  type Result = ();

  fn handle(&mut self, msg: RegisterConnection, _: &mut Context<Self>) {
    info!("Registering connection for endpoint: {}, id: {}", msg.endpoint, msg.id);
    // self.insert(&msg.endpoint, msg.id, msg.connection);
  }
}

impl Handler<PublishMessage> for RedisActor {
  type Result = ();

  fn handle(&mut self, event: PublishMessage, _ctx: &mut Self::Context) {
    // if let Some(endpoint_sockets) = self.clients.get(&event.endpoint) {
    //   for (_, addr) in endpoint_sockets {
    //     addr.do_send(WebsockeMessageEvent {
    //       message: event.message.clone(),
    //     });
    //   }
    // }

    // publish it to redis channel
  }
}