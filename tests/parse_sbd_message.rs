use chrono::{TimeZone, Utc};
use sbd_lib::{mo, mt, InformationElement, Message};
use std::io::Cursor;

#[test]
fn sbd_mo_message_without_location() {
    let message = Message::from_path("data/0-mo.sbd").unwrap();

    let msg = Message::new(
        mo::Header {
            auto_id: 1894516585,
            session_status: mo::SessionStatus::Ok,
            imei: *b"300234063904190",
            momsn: 75,
            mtmsn: 0,
            time_of_session: Utc.timestamp(1436465708, 0),
        }
        .into(),
        vec![
            116, 101, 115, 116, 32, 109, 101, 115, 115, 97, 103, 101, 32, 102, 114, 111, 109, 32,
            112, 101, 116, 101,
        ],
        None,
        vec![],
    );
    assert_eq!(msg, message);
}

#[test]
fn sbd_mo_message_wit_location() {
    use std::fs::File;
    let file = File::open("data/1-mo-location.sbd").unwrap();
    let iei = InformationElement::read(file).unwrap();

    let message = Message::create(iei).unwrap();
    let location = message.location().unwrap();

    let loc = mo::LocationInformation::new(0, (60, 5132), (29, 14176), Some(93));

    assert_eq!(loc, location);

    let mut buff = vec![];
    message.write_to(&mut buff).unwrap();
    let read_back = Message::read_from(Cursor::new(buff)).unwrap();
    assert_eq!(message, read_back);
}

#[test]
fn sbd_responce_for_mt() {
    use std::fs::File;
    let file = File::open("data/resp.sbd").unwrap();
    let iei = InformationElement::read(file).unwrap();
    let resp = InformationElement::Status(
        mt::ConfirmationStatus {
            message_id: 287454020,
            imei: *b"300434060009290",
            auto_id: 3064195606,
            status: 1,
        }
        .into(),
    );
    assert_eq!(resp, iei[0]);
}

#[test]
fn sbd_mo_message_wit_location_big_data() {
    use std::fs::File;
    let file = File::open("data/data.sbd").unwrap();
    let iei = InformationElement::read(file).unwrap();
    let message = Message::create(iei).unwrap();
    let msg = Message::new(
        mo::Header {
            auto_id: 26502124,
            session_status: mo::SessionStatus::Ok,
            imei: *b"300434069104350",
            momsn: 545,
            mtmsn: 0,
            time_of_session: Utc.timestamp(1587546484, 0),
        }
        .into(),
        vec![
            10, 16, 18, 138, 173, 180, 242, 224, 34, 215, 130, 227, 62, 42, 16, 32, 165, 139, 16,
            1, 24, 129, 147, 128, 245, 5, 32, 129, 138, 188, 139, 1, 40, 8, 48, 75, 56, 156, 167,
            215, 186, 4, 64, 136, 224, 153, 172, 2, 72, 152, 1,
        ],
        Some(mo::LocationInformation::new(
            0,
            (59, 48741),
            (31, 30844),
            Some(3),
        )),
        vec![],
    );

    assert_eq!(msg, message);
}
