#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::fmt;
use std::iter;
use std::iter::Chain;
use std::thread;
use std::time::Duration;

mod model;

use crate::model::*;

fn main() {
    let comments = Comments::for_user("shim__");
    for comment in comments {
        dbg!(comment);
        thread::sleep(Duration::from_millis(100));
    }
    println!("Hello, world!");
}

/*fn fetch_comments<'a, T: AsRef<str> + fmt::Display>(
    redditor: &'a T,
    from: Option<String>,
) -> std::iter::Chain<
    std::result::Result<
        std::vec::Vec<model::Comment>,
        std::boxed::Box<(dyn std::error::Error + 'static)>,
    >,
    std::vec::Vec<model::Comment>,
> {
    fn request_paged(url: &str) -> Result<(Vec<Comment>, Option<String>), Box<Error>> {
        let comment_json = reqwest::get(url)?.text()?;

        let continuation: Option<String> = unimplemented!();
        let comments: Vec<Comment> = unimplemented!();
        return Ok((comments, continuation));
    }
    let page = request_paged(&format!("https://reddit.com/user/{}.json", redditor));
    let next: std::iter::Chain<
        std::result::Result<
            std::vec::Vec<model::Comment>,
            std::boxed::Box<(dyn std::error::Error + 'static)>,
        >,
        std::vec::Vec<model::Comment>,
    > = if let Ok((_, cont)) = page {
        fetch_comments(redditor, cont)
    } else {
        unimplemented!() //iter::once(Vec::with_capacity(0).iter()).chain([].into_iter().map(|_| fetch_comments("never_gonna_happen", None)))
    };
    iter::once([()].into_iter().map(|_| page)).chain([()].into_iter().flat_map(|_| next))
}*/

fn poll(interval: Duration, count: Option<u32>) {
    let mut it: u32 = 0;
    loop {
        thread::sleep(interval);
        it += 1;
        if count.map(|c| c < it).unwrap_or(false) {
            break;
        }
    }
}
