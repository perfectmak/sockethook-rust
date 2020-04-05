#[macro_use]
extern crate log;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::json;
use structopt::StructOpt;
use actix::prelude::*;
use actix_web::{http::header, web, App, HttpServer, HttpRequest, HttpResponse, Error, Responder};
use actix_cors::Cors;
use actix_web_actors::ws;
use websocket::WebsocketConnection;
use app_data::AppData;
use messages::{RegisterConnection, PublishMessage, Shutdown};
use redis::RedisActor;

mod websocket;
mod messages;
mod redis;
mod app_data;

const HOOK_PATH: &'static str = "/hook{endpoint:/.*}";
const SOCKET_PATH: &'static str = "/socket{endpoint:/.*}";

/// Represent structure of json data that is pushed to all endpoint websockets 
#[derive(Debug, Serialize, Deserialize)]
struct Message<Data> 
where Data: Sized {
  headers: HashMap<String, String>,
  endpoint: String,
  data: Data,
}

fn handle_hooks(
  app_data: web::Data<Addr<AppData>>,
  endpoint: web::Path<String>,
  req: HttpRequest,
  str_data: Option<String>,
) -> impl Responder {
  let mut headers: HashMap<String, String> = HashMap::new();
  for (header_name, header_value) in req.headers() {
    headers.insert(
      header_name.to_string(),
      header_value.to_str().unwrap_or("invalid_asci_value_set").to_owned(),
    );
  }

  let message = Message {
    headers,
    endpoint: endpoint.to_string(),
    data: str_data.unwrap_or_default(),
  };
  let message = json!(message).to_string();
  
  // publish message to all websockets
  app_data.get_ref().do_send(PublishMessage {
    endpoint: endpoint.to_string(),
    message
  });

  HttpResponse::Ok()
}

fn handle_client(
  app_data: web::Data<Addr<AppData>>,
  endpoint: web::Path<String>,
  req: HttpRequest,
  stream: web::Payload,
) -> Result<HttpResponse, Error> {
  let connection = WebsocketConnection::new(endpoint.to_string(), app_data.get_ref().clone());
  let connection_id = connection.id;
  let (addr, res) = ws::start_with_addr(connection, &req, stream)?;

  app_data.do_send(RegisterConnection{
    id: connection_id,
    endpoint: endpoint.to_string(),
    connection: addr,
  });

  Ok(res)
}

/// Cli Arguments
#[derive(StructOpt)]
#[structopt(name = "sockethook", version = "1.0.0", author = "Perfect Makanju")]
struct Args {
  /// Sets the port to listen on for incoming requests. Default (1234)
  #[structopt(short = "p", long = "port", default_value = "1234")]
  port: String,
  /// Sets the network address this program should bind to. Default (0.0.0.0)
  #[structopt(short = "a", long = "address", default_value = "0.0.0.0")]
  address: String,
  /// Set a redis connection string for coordinating multiple instances of sockethoot.
  #[structopt(short = "r", long = "redis")]
  redis_url: Option<String>,
}

fn main() -> std::io::Result<()> {
  env_logger::builder()
    .filter_module("sockethook", log::LevelFilter::Info)
    .init();

  let args = Args::from_args();
  let sys = System::new("sockethook");
  let app_data = match args.redis_url { 
    Some(redis_url) => {
      AppData::new_with_redis(redis_url.clone())
    },
    None => AppData::new(),
  }.start();

  let app_data_copy = app_data.clone();
  let addr_str = format!("{}:{}", args.address, args.port);

  // handle SIGINT
  ctrlc::set_handler(move || {
    info!("Received SIGINT, shutting down");
    let error_occurred = app_data_copy.send(Shutdown)
      .wait()
      .is_err();
    if error_occurred {
      error!("Error handling SIGINT");
    }
  }).expect("Error setting SIGINT handler");

  // Startup server
  HttpServer::new(move || {
    App::new()
      .wrap(
        Cors::new()
          .allowed_origin("*")
          .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
          .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
          .max_age(3600),
      )
      .data(app_data.clone())
      .service(web::resource(HOOK_PATH).route(web::post().to(handle_hooks)))
      .service(web::resource(SOCKET_PATH).route(web::get().to(handle_client)))
  })
  .bind(addr_str.clone())
  .unwrap()
  .start();

  info!("Sockethook is ready and listening at {} ✅", &addr_str);

  sys.run()
}
