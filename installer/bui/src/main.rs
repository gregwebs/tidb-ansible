extern crate failure;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate raii_change_tracker;
extern crate bui_backend;
#[cfg(feature = "bundle_files")]
extern crate includedir;
#[cfg(feature = "bundle_files")]
extern crate phf;
extern crate serde_json;
extern crate clap;
extern crate hyper;
extern crate dotenv;
extern crate futures;
extern crate tokio_core;

use failure::Error;

use std::sync::{Arc, Mutex};

use raii_change_tracker::DataTracker;
use bui_backend::highlevel::{BuiAppInner, create_bui_app_inner};

use futures::{Future, Stream};

extern crate tidb_installer;
#[macro_use]
extern crate serde_derive;
use tidb_installer::args::{jwt_secret, is_loopback, address, app_arguments};
use tidb_installer::command;


// Include the files to be served and define `fn get_default_config()`.
include!(concat!(env!("OUT_DIR"), "/public.rs")); // Despite slash, this does work on Windows.

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Shared {
    pub error: String,
    pub running: Option<command::RunningCommand>,
}

/// The structure that holds our app data
struct MyApp {
    inner: BuiAppInner<Shared>,
}

impl MyApp {
    /// Create our app
    fn new(secret: &[u8], addr: &std::net::SocketAddr, config: Config) -> Self {

        // Create our shared state.
        let shared_store = Arc::new(Mutex::new(DataTracker::new(Shared {
                                                error: "".into(),
                                                running: None,
                                            })));

        // Create `inner`, which takes care of the browser communication details for us.
        let chan_size = 10;
        let (_, mut inner) =
            create_bui_app_inner(&secret, shared_store, &addr, config, chan_size, "/events");

        // Make a clone of our shared state Arc which will be moved into our callback handler.
        let tracker_arc2 = inner.shared_arc().clone();

        // Create a Stream to handle callbacks from clients.
        let callback_rx_future = inner
            .add_callback_listener(10) // max number of callbacks to buffer
            .for_each(move |msg| {

                // This closure is the callback handler called whenever the
                // client browser sends us something.

                // Get access to our shared state so we can modify it based on
                // the browser's callback.
                let mut shared = tracker_arc2.lock().unwrap();

                // All callbacks have the `name` field.
                println!("command name: {}", msg.name);
                match msg.name.as_ref() {
                    "just" => {
                        // Take the generic `args` and convert it to a String.
                        match serde_json::from_value::<String>(msg.args) {
                            Ok(just_recipe) => {
                                println!("just {}", just_recipe);
                                match command::background(just_recipe) {
                                    Ok(running) => {
                                        let mut state = shared.as_tracked_mut();
                                        state.running = Some(running);
                                    }
                                  , Err(e) => {error!("could not run command: {:?}", e);}
								}
                            },
                            Err(e) => {
                                error!("could not cast json value to String: {:?}", e);
                            },
                        };
                    },
                    name => {
                        // This is an error case. Log it. (And do not take down the server.)
                        error!("callback with unknown name: {:?}", name);
                    },
                }
                futures::future::ok(())
            });

        // Add our future into the event loop created by hyper.
        inner.hyper_server().handle().spawn(callback_rx_future);

        // Return our app.
        MyApp { inner: inner }
    }

    /// Get a handle to our event loop.
    fn handle(&self) -> tokio_core::reactor::Handle {
        self.inner.hyper_server().handle()
    }

    /// Consume self and run forever.
    fn run(self) -> std::result::Result<(), hyper::Error> {
        self.inner.into_hyper_server().run()
    }
}

fn run() -> Result<(),Error> {

    // Set environment variables from `.env` file, if it exists.
    dotenv::dotenv().ok();

    // Setup logging based on level in RUST_LOG environment variable.
    env_logger::init();

    // Parse our command-line arguments.
    let matches = app_arguments().get_matches();
    let http_server_addr = address(&matches);
    let required = !is_loopback(&http_server_addr);
    let secret = jwt_secret(&matches, required)?;

    // This `get_default_config()` function is created by bui_backend_codegen
    // and is pulled in here by the `include!` macro above.
    let config = get_default_config();

    // Create our app.
    let my_app = MyApp::new(&secret, &http_server_addr, config);

    // Clone our shared data to move it into a closure later.
    let tracker_arc = my_app.inner.shared_arc().clone();

    // Get a handle to our event loop.
    let handle = my_app.handle();

    // Create a stream to periodically call our closure
    let interval_stream: tokio_core::reactor::Interval =
        tokio_core::reactor::Interval::new(std::time::Duration::from_millis(250), &handle)
            .unwrap();

    match command::background_exit_code() {
        Ok(_) => {}
      , Err(_) => {
          println!("resetting background command info");
          command::reset_background().expect("reset background output");
        }
    }
    let stream_future = interval_stream
        .for_each(move |_| {
            // Update the command output
            match command::update_running_command()? {
                None => {}
                Some(running) => {
                  let mut shared_store = tracker_arc.lock().unwrap();
                  let mut shared = shared_store.as_tracked_mut();
                  shared.running = Some(running);
                }
            }
            Ok(())
        })
        .map_err(|e| {
                     error!("interval error {:?}", e);
                     ()
                 });

    // Put our stream into our event loop.
    my_app.handle().spawn(stream_future);

    println!("Listening on http://{}", http_server_addr);

    // Run our app.
    my_app.run()?;
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {},
        Err(e) => {
            error!("{}, {}", e.cause(), e.backtrace());
            std::process::exit(1);
        }
    }
}
