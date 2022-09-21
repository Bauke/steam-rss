//! # Steam RSS
//!
//! > **Get RSS feeds for Steam games.**
//!
//! *AGPL-3.0-or-later*

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use std::{thread::sleep, time::Duration};

use {
  clap::Parser,
  color_eyre::{install, Result},
  indicatif::{ProgressBar, ProgressStyle},
};

/// CLI arguments struct using [`clap`]'s Derive API.
#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
  /// A game's AppID, can be used multiple times.
  #[clap(short, long)]
  pub appid: Vec<usize>,

  /// The time in milliseconds to sleep between HTTP requests.
  #[clap(short, long, default_value = "250")]
  pub timeout: u64,

  /// Verify potential feeds by downloading them and checking if they return XML.
  #[clap(short, long)]
  pub verify: bool,
}

fn main() -> Result<()> {
  install()?;

  let args = Args::parse();
  let timeout = Duration::from_millis(args.timeout);

  let ureq_agent = ureq::AgentBuilder::new()
    .user_agent("Steam Feeds (https://git.bauke.xyz/Bauke/steam-rss)")
    .build();
  let mut potential_feeds = vec![];
  let mut feeds_to_output = vec![];

  for appid in args.appid {
    potential_feeds
      .push(format!("https://steamcommunity.com/games/{appid}/rss/"));
  }

  if args.verify {
    let progress = ProgressBar::new(potential_feeds.len().try_into()?)
      .with_style(ProgressStyle::with_template("Verifying {pos}/{len} {bar}")?);

    for potential_feed in potential_feeds {
      let response = ureq_agent.get(&potential_feed).call()?;
      if response.content_type() == "text/xml" {
        feeds_to_output.push(potential_feed);
      }

      sleep(timeout);
      progress.inc(1);
    }
  } else {
    feeds_to_output = potential_feeds;
  }

  for feed in feeds_to_output {
    println!("{feed}");
  }

  Ok(())
}
