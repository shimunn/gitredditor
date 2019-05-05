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
    buffer: Option<Box<Iterator<Item = Result<Comment, Box<Error>>>>>,
}

impl Comments {
    fn new<T: ToString>(url: T) -> Comments {
        let url = url.to_string();
        Comments {
            url: url,
            continuation: None,
            buffer: None,
        }
    }
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

        if let (Some(ref mut buffer), ref mut continuation, ref url) =
            (&mut self.buffer, &mut self.continuation, &self.url)
        {
            if let Some(comment) = buffer.next() {
                Some(comment)
            } else {
                continuation.clone().and_then(|cont| {
                    let page = request_paged(&(url.to_string() + "?after=" + &cont[..]));
                    match page {
                        Ok((comments, cont)) => {
                            **continuation = cont;
                            *buffer = Box::new(comments.into_iter().map(Ok))
                        }
                        Err(e) => *buffer = Box::new(iter::once(Err(e))),
                    };
                    buffer.next()
                })
            }
        } else {
            let page = request_paged(&self.url);
            match page {
                Ok((comments, cont)) => {
                    self.continuation = cont;
                    self.buffer = Some(Box::new(comments.into_iter().map(Ok)))
                }
                Err(e) => self.buffer = Some(Box::new(iter::once(Err(e)))),
            };
            self.buffer.as_mut().and_then(|it| it.next())
        }
    }
}
