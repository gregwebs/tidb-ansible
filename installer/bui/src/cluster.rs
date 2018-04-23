use std;
use command;

pub fn get_cluster_info() -> Result<Option<Vec<String>>, std::io::Error> {
  let hosts = vec!["monitor-0", "tidb-0", "tikv-0", "tikv-1", "tikv-2", "pd-0", "pd-1", "pd-2"];
  let mut outputs = vec![];
  for host in hosts.iter() {
    let output = command::run("kubectl", vec!["exec", host, "--", "systemctl", "status"])?;
    // TODO: this is a hack to short-circuit when there are not hosts yet
    if output.is_empty() {
      return Ok(None);
    }
    outputs.push(output);
  }
  Ok(Some(outputs))
}


/*
mod http {
	extern crate futures;
	extern crate hyper;
	extern crate tokio_core;

	//use cluster::futures::future::{Future};
	use std::io::{Error, ErrorKind};
	use cluster::http::hyper::client::{Client, HttpConnector};
	use cluster::http::hyper::{Body, Uri};

	pub enum HttpError {
	  HyperError(hyper::error::Error),
	  IOError(Error),
	}
	impl From<Error> for HttpError {
		fn from(err: Error) -> HttpError { HttpError::IOError(err) }
	}
	impl From<hyper::error::Error> for HttpError {
		fn from(err: hyper::error::Error) -> HttpError { HttpError::HyperError(err) }
	}
	impl From<HttpError> for Error {
		fn from(err: HttpError) -> Error {
			match err {
				HttpError::IOError(ioe) => ioe
			  , HttpError::HyperError(any_err) =>
				  Error::new(ErrorKind::Other, format!("{:}", any_err))
			}
		}
	}

	pub fn update_cluster_info(client: Client<HttpConnector, Body>)
	  -> hyper::client::FutureResponse {
	 // -> futures::AndThen<hyper::client::FutureResponse, std::result::Result<(), hyper::Error>> {
	  let uri : Uri = "http://localhost:7000/api/v1/pods".parse().unwrap();
	  println!("making kube request");
	  client.get(uri)/*.and_then(|res| {
		  println!("Response: {}", res.status());
		Ok(())
	  })
	*/
	}
}
*/
