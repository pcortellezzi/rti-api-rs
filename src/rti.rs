pub mod messages;

#[allow(clippy::all, clippy::pedantic)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/rti.rs"));
}

pub use generated::*;