mod client_request;
mod database;
mod server_request;

use common::database_packet::*;
use common::ids::*;
use database::*;

fn main() {
    common::logger::Logger::init();
    common::load_data();
    Database::start();
}
