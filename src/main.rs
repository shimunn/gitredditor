#[macro_use]
extern crate serde_derive;

use git2::{Repository, Signature, Time as GitTime};
use serde_json::to_string_pretty;
use std::error::Error;
use std::fs::{create_dir_all, read_to_string, write as fs_write};
use std::iter::IntoIterator;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
mod model;
mod opts;

use crate::model::*;
use crate::opts::*;

fn main() {
    let opts = Opts::from_args();
    let comments = Comments::for_user(&opts.redditor);

    println!(
        "Hello, world! {:?}",
        update(
            &opts.repo.unwrap_or(PathBuf::from("repo")),
            comments.take(opts.fetch).flatten(),
            &opts.redditor,
            &("reddit.com/u/".to_owned() + &opts.redditor),
            (opts.threshold, opts.thresholdp)
        )
    );
}

fn update(
    repo: &PathBuf,
    current: impl IntoIterator<Item = Comment>,
    redditor: &str,
    email: &str,
    threshold: (u32, u8),
) -> Result<usize, Box<Error>> {
    let comment_path = |c: &Comment| {
        let mut p = repo.clone();
        for s in c.permalink.split('/') {
            p.push(s);
        }
        p.set_extension("json");
        p
    };
    let mut updated: usize = 0;
    let git = Repository::open(&repo)?;
    let sig = match git.signature() {
        Err(_) => Signature::now(redditor, email),
        sig => sig,
    }?;
    let mut index = git.index()?;
    index.read(false)?;
    let (threshold, threshold_percent) = threshold;
    let threshold_percent = f32::from(threshold_percent);
    let head = || match git.head() {
        Ok(head) => match head.target() {
            Some(oid) => git.find_commit(oid).ok(),
            None => None,
        },
        Err(_) => None,
    };
    let mut parent = head();
    for comment in current.into_iter() {
        let path = comment_path(&comment);
        let path_rel = || {
            let mut p = PathBuf::from(&comment.permalink[1..(comment.permalink.len() - 1)]);
            p.set_extension("json");
            p
        };
        let mut commit_msg = String::new();
        let mut commit_timestamp: Option<SystemTime> = None;
        let changed = if (&path).exists() {
            let content = read_to_string(&path)?;
            let old: Comment = serde_json::from_str(&content)?;
            let delta = CommentDelta::from(&old, &comment)
                .into_iter()
                .filter(|d| match d {
                    CommentDelta::Votes(change) => {
                        commit_timestamp = Some(SystemTime::now());
                        change.abs() as u32 > threshold
                            && change.abs() as f32 > old.score as f32 * (threshold_percent / 100.0)
                    }
                    _ => true,
                })
                .collect::<Vec<_>>();
            if !delta.is_empty() {
                fs_write(&path, to_string_pretty(&comment)?)?;
                for msg in delta.iter() {
                    commit_msg.push_str(&msg.to_string());
                    commit_msg.push('\n');
                }
                updated += 1;
                true
            } else {
                false
            }
        } else {
            create_dir_all((&path).parent().unwrap())?;
            fs_write(&path, to_string_pretty(&comment)?)?;
            commit_msg = CommentDelta::New.to_string();
            updated += 1;
            true
        };
        if changed {
            index.add_path(&path_rel())?;
            let tree_id = index.write_tree()?;
            let tree = git.find_tree(tree_id)?;

            let time = commit_timestamp.unwrap_or_else(|| comment.last_update());

            let sig_backdate = Signature::new(sig.name().unwrap(), sig.email().unwrap(), &{
                let dur = dbg!(time).duration_since(UNIX_EPOCH).unwrap();
                GitTime::new(dur.as_secs() as i64, dur.subsec_millis() as i32)
            })?;

            println!("Commiting: {}:\n{}", comment.id, commit_msg);
            let commit = git.commit(
                Some("HEAD"),
                &sig_backdate,
                &sig,
                &commit_msg,
                &tree,
                &parent.iter().collect::<Vec<_>>()[..], //TODO: this isnt ideal
            )?;
            parent = Some(git.find_commit(commit)?);
        }
    }
    index.write()?;

    Ok(updated)
}
