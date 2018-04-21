use std;
use std::net::ToSocketAddrs;
use clap;
use failure::{Error};


pub fn address( matches: &clap::ArgMatches ) -> std::net::SocketAddr {
    let host = matches.value_of( "host" ).unwrap();
    let port = matches.value_of( "port" ).unwrap();
    format!( "{}:{}", host, port ).to_socket_addrs().unwrap().next().unwrap()
}

pub fn is_loopback(addr_any: &std::net::SocketAddr) -> bool {
    match addr_any {
        &std::net::SocketAddr::V4(addr) => addr.ip().is_loopback(),
        &std::net::SocketAddr::V6(addr) => addr.ip().is_loopback(),
    }
}

/// Parse the JWT secret from command-line args or environment variables.
pub fn jwt_secret(matches: &clap::ArgMatches, required: bool) -> Result<Vec<u8>,Error> {
    match matches
        .value_of("JWT_SECRET")
        .map(|s| s.into())
        .or(std::env::var("JWT_SECRET").ok())
        .map(|s| s.into_bytes())
    {
        Some(secret) => Ok(secret),
        None => {
            if required {
                Err(format_err!("The --jwt-secret argument must be passed or the JWT_SECRET environment \
                variable must be set when not using loopback interface."))
            } else {
                Ok(b"jwt_secret".to_vec())
            }
        }
    }
}

pub fn app_arguments<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("CARGO_PKG_NAME")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(clap::Arg::with_name("JWT_SECRET")
                 .long("jwt-secret")
                 .help("Specifies the JWT secret. Falls back to the JWT_SECRET \
                environment variable if unspecified.")
                 .global(true)
                 .takes_value(true))
        .arg(clap::Arg::with_name( "host" )
                .long( "host" )
                .help( "Bind the server to this address")
                .default_value("localhost")
                .value_name( "HOST" )
                .takes_value( true ))
        .arg(clap::Arg::with_name( "port" )
                .long( "port" )
                .help( "Bind the server to this port, default 3410" )
                .default_value("3410")
                .value_name( "PORT" )
                .takes_value( true )
        )
}
