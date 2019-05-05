use std::path::PathBuf;
pub use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "gitredditor")]
pub struct Opts {
    #[structopt(short = "f", long = "fetch", default_value = "20")]
    pub fetch: usize,

    #[structopt(short = "r", long = "redditor")]
    pub redditor: String,

    #[structopt(parse(from_os_str))]
    pub repo: Option<PathBuf>,
}
