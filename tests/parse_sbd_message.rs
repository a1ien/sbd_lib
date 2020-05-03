use chrono::{TimeZone, Utc};
use sbd_lib::{mo, mt, InformationElement, Message};
use std::io::Cursor;

#[test]
fn sbd_mo_message_without_location() {
    let message = Message::from_path("data/0-mo.sbd").unwrap();
    println!("{:?}", message);
}

#[test]
fn sbd_mo_message_wit_location() {
    let message = Message::from_path("data/1-mo-location.sbd").unwrap();
    println!("{:?}", message);
}

#[test]
fn sbd_responce_for_mt() {
    use std::fs::File;
    let file = File::open("data/resp.sbd").unwrap();
    let iei = InformationElement::read(file);
    println!("{:?}", iei);
}
