#![allow(unexpected_cfgs)]

use ruma_common::api::response;

#[derive(PartialEq)] // Make sure attributes work
#[response]
pub struct Response {
    pub flag: bool,
}

fn main() {
    let res1 = Response { flag: false };
    let res2 = res1.clone();

    assert_eq!(res1, res2);
}
