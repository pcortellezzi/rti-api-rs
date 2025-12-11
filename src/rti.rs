pub mod messages;

mod proto {
    #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
    include!(concat!(env!("OUT_DIR"), "/rti.rs"));
}

pub use proto::*;