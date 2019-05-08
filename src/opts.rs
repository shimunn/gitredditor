use std::path::PathBuf;
pub use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "gitredditor")]
pub struct Opts {
    #[structopt(
        short = "f",
        long = "fetch",
        default_value = "20",
        env = "GITREDDITOR_CNT"
    )]
    pub fetch: usize,

    #[structopt(
        short = "t",
        long = "threshold",
        default_value = "5",
        env = "GITREDDITOR_TH"
    )]
    pub threshold: u32,

    #[structopt(
        short = "p",
        long = "threshold-percent",
        default_value = "5",
        env = "GITREDDITOR_THP"
    )]
    pub thresholdp: u8,

    #[structopt(short = "r", long = "redditor", env = "GITREDDITOR_U")]
    pub redditor: String,

    #[structopt(parse(from_os_str))]
    pub repo: Option<PathBuf>,
}
