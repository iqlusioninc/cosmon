//! HTTP collector

use crate::monitor::message;
use warp::Filter;

/// Run the HTTP collector
pub fn run() {
    // POST /collector
    let collector = warp::post2()
        .and(warp::path("collector"))
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::json())
        .map(|envelope: message::Envelope| {
            dbg!(envelope);
            warp::reply()
        });

    warp::serve(collector).run(([127, 0, 0, 1], 3030));
}
