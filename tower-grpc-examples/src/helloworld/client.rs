#![deny(warnings, rust_2018_idioms)]

use futures::Future;
use hyper::client::connect::{Destination, HttpConnector};
use tower_grpc::Request;
use tower_hyper::{client, util};
use tower_util::MakeService;

use std::error::Error;


// use tower_grpc::codegen::client::grpc::GrpcService;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

type Client = hello_world::client::Greeter<tower_request_modifier::RequestModifier<tower_hyper::client::Connection<tower_grpc::BoxBody>, tower_grpc::BoxBody>>;

fn make_client(uri: http::Uri) -> Result<Box<dyn Future<Item=Client, Error=tower_grpc::Status> + Send>, Box<dyn Error>> {
// where T:  tower_service::Service<http::Request<B>> {

// Result<Box<dyn Future<Item=hello_world::client::Greeter<T>, Error=tower_grpc::Status> + Send>, Box<dyn Error>> 
//   where T: GrpcService<R>,
//            tower_grpc::codegen::client::grpc::unary::Once<M>: tower_grpc::client::Encodable<R> {
    let dst = Destination::try_from_uri(uri.clone())?;
    let connector = util::Connector::new(HttpConnector::new(4));
    let settings = client::Builder::new().http2_only(true).clone();
    let mut make_client = client::Connect::with_builder(connector, settings);

    let say_hello = make_client
        .make_service(dst)
        .map_err(|e| panic!("connect error: {:?}", e))
        .and_then(move |conn| {
            use crate::hello_world::client::Greeter;

            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .unwrap();

            // Wait until the client is ready...
            Greeter::new(conn).ready()
        });
    Ok(Box::new(say_hello))
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let _ = ::env_logger::init();

    let uri: http::Uri = format!("http://[::1]:50051").parse()?;

    let say_hello = make_client(uri)?
        .and_then(|mut client| {
            use crate::hello_world::HelloRequest;

            client.say_hello(Request::new(HelloRequest {
                name: "What is in a name?".to_string(),
            }))
        })
        .and_then(|response| {
            println!("RESPONSE = {:?}", response);
            Ok(())
        })
        .map_err(|e| {
            println!("ERR = {:?}", e);
        });

    tokio::run(say_hello);
    Ok(())
}
