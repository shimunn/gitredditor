use serde::{Deserialize, Serialize};
use std::error::Error;

use std::iter;
use std::iter::Chain;

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    score: u32,
    id: String,
    created: u64,
    permalink: String,
    edited: bool,
}

pub struct Comments {
    pub url: String,
    continuation: Option<String>,
    buffer: Box<Iterator<Item = Result<Comment, Box<Error>>>>,
}

impl Iterator for Comments {

    type Item = Result<Comment, Box<Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        fn request_paged(url: &str) -> Result<(Vec<Comment>, Option<String>), Box<Error>> {
            let comment_json = reqwest::get(url)?.text()?;

            let continuation: Option<String> = unimplemented!();
            let comments: Vec<Comment> = unimplemented!();
            return Ok((comments, continuation));
        }

        if let Some(comment) = self.buffer.next() {
            Some(comment)
        } else {
            self.continuation.clone().and_then(|cont| {
                let page = request_paged(&self.url);
                match page {
                    Ok((comments, cont)) => {
                        self.continuation = cont;
                        self.buffer = Box::new(comments.into_iter().map(Ok))
                    }
                    Err(e) => self.buffer = Box::new(iter::once(Err(e))),
                };
                self.buffer.next()
            })
        }
    }
}
