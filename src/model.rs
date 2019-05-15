use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Comment {
    pub score: i32,
    pub id: String,
    pub created: f64,
    pub permalink: String,
    #[serde(default)]
    pub body: String,
    #[serde(deserialize_with = "false_or_val")]
    pub edited: Option<u64>,
}

impl Comment {
    pub fn last_update(&self) -> SystemTime {
        let created = UNIX_EPOCH + Duration::from_secs(self.created as u64);
        let edited = self
            .edited
            .filter(|e| e > &1)
            .map(|e| UNIX_EPOCH + Duration::from_secs(e));
        if let Some(edited) = edited {
            if edited > created {
                edited
            } else {
                created
            }
        } else {
            created
        }
    }
}

pub enum CommentDelta {
    Votes(i32),
    Content,
    New,
}

impl fmt::Display for CommentDelta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CommentDelta::*;
        match self {
            Votes(d) if d > &0 => write!(f, "Received +{} upvotes", d),
            Votes(d) if d < &0 => write!(f, "Received {} downvotes", d),
            Votes(_) => write!(f, "You shouln't see this one, if you do check the source"),
            Content => write!(f, "Edited"),
            New => write!(f, "Created"),
        }
    }
}

impl CommentDelta {
    pub fn from(a: &Comment, b: &Comment) -> Vec<CommentDelta> {
        assert_eq!(a.id, b.id);
        //Assume that most comments don't change
        let mut delta = Vec::with_capacity(0);
        use CommentDelta::*;
        if a.score != b.score {
            delta.push(Votes(b.score - a.score))
        }

        if a.body != b.body {
            delta.push(Content)
        }
        delta
    }
}

fn false_or_val<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;
    use serde_json::Value::*;
    Ok(match val {
        Number(num) => num.as_u64(),
        Bool(true) => Some(1),
        _ => None,
    })
}

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct ListItem<T: for<'a> Deserialize<'a>> {
 kind: String,
 data: T,
}*/

pub struct Comments {
    pub url: String,
    continuation: Option<String>,
    buffer: Option<Result<Box<Iterator<Item = Result<Comment, Box<Error>>>>, Box<Error>>>,
    no_it: usize,
}

impl Comments {
    pub fn new<T: ToString>(url: T) -> Comments {
        let url = url.to_string();
        Comments {
            url: url,
            continuation: None,
            buffer: None,
            no_it: 0,
        }
    }

    pub fn for_user<T: ToString>(name: T) -> Comments {
        Self::new(format!(
            "https://www.reddit.com/user/{}.json",
            name.to_string()
        ))
    }
}

impl Iterator for Comments {
    type Item = Result<Comment, Box<Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        fn request_paged(url: &str) -> Result<(Vec<Comment>, Option<String>), Box<Error>> {
            let comment_json = reqwest::get(dbg!(url))?.text()?;
            let comment_json: Value = serde_json::from_str(&comment_json)?;
            let data: &Value = &comment_json["data"];
            let continuation: Option<String> = match &data["after"] {
                Value::String(after) if after.len() > 0 => Some(after.clone()),
                _ => None,
            };
            //Kinda ugly .clone()
            //let comments: Vec<ListItem<Comment>> = serde_json::from_value(data["children"].clone())?;
            if let Some(children) = data["children"].as_array() {
                let comments = children
                    .iter()
                    .map(|li| serde_json::from_value::<Comment>(li["data"].clone()))
                    .collect::<Result<Vec<_>, _>>();
                comments
                    .map(|comments| (comments, continuation))
                    .map_err(|e| e.into())
            } else {
                Err("Parse err".into())
            }
            //return Ok((comments.iter().map(|li| li.data).collect(), continuation));
        }

        let (ref mut buffer, ref mut continuation, ref mut no_it, ref url) = (
            &mut self.buffer,
            &mut self.continuation,
            &mut self.no_it,
            &self.url,
        );
        let mut fetch_lot = |buffer: &mut Option<
            Result<Box<Iterator<Item = Result<Comment, Box<Error>>>>, Box<Error>>,
        >,
                             continuation: &mut Option<String>| {
            let page = match continuation {
                Some(ref cont) => request_paged(
                    &(url.to_string() + "?after=" + &cont[..] + {
                        if **no_it > 5 {
                            thread::sleep(Duration::from_millis(100));
                            "&limit=100"
                        } else {
                            ""
                        }
                    }),
                ),
                None => request_paged(url),
            };
            **no_it += 1;
            match page {
                Ok((comments, cont)) => {
                    *continuation = cont;
                    *buffer = Some(Ok(Box::new(comments.into_iter().map(Ok))));
                }
                Err(e) => *buffer = Some(Err(e)),
            };
        };
        match buffer {
            None => {
                //Init
                fetch_lot(buffer, continuation);
            }
            Some(Err(_)) => return None,
            _ => (),
        }

        match buffer.as_mut().map(|buf| buf.as_mut().map(|it| it.next())) {
            Some(Ok(Some(Err(e)))) => {
                **buffer = Some(Err(e.description().into()));
                Some(Err(e))
            }
            Some(Ok(Some(Ok(i)))) => Some(Ok(i)),
            Some(Ok(None)) if continuation.is_some() => {
                fetch_lot(buffer, continuation);
                self.next()
            }
            _ => None,
        }
    }
}
