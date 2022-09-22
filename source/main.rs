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
  regex::Regex,
};

/// CLI arguments struct using [`clap`]'s Derive API.
#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
  /// A game's AppID, can be used multiple times.
  #[clap(short, long)]
  pub appid: Vec<usize>,

  /// Output the feeds as OPML.
  #[clap(long)]
  pub opml: bool,

  /// The time in milliseconds to sleep between HTTP requests.
  #[clap(short, long, default_value = "250")]
  pub timeout: u64,

  /// Verify potential feeds by downloading them and checking if they return XML.
  #[clap(short, long)]
  pub verify: bool,

  /// A game's store URL, can be used multiple times.
  #[clap(long)]
  pub url: Vec<String>,
}

/// A simple feed struct.
#[derive(Debug)]
pub struct Feed {
  /// The CLI option that was used for this feed.
  pub option: FeedOption,

  /// The text to use for the feed in the OPML output.
  pub text: Option<String>,

  /// The URL of the feed.
  pub url: String,
}

/// An enum for [`Feed`]s for which option was used in the CLI.
#[derive(Debug, Eq, PartialEq)]
pub enum FeedOption {
  /// `-a, --appid <APPID>` was used.
  AppID,

  /// `--url <URL>` was used.
  Url,
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

  let store_url_regex =
    Regex::new(r"(?i)^https?://store.steampowered.com/app/(?P<appid>\d+)")?;

  for appid in args.appid {
    potential_feeds.push(Feed {
      option: FeedOption::AppID,
      text: Some(format!("Steam AppID {appid}")),
      url: appid_to_rss_url(appid),
    });
  }

  for url in args.url {
    let appid = store_url_regex
      .captures(&url)
      .and_then(|captures| captures.name("appid"))
      .and_then(|appid_match| appid_match.as_str().parse::<usize>().ok());
    if let Some(appid) = appid {
      potential_feeds.push(Feed {
        option: FeedOption::Url,
        text: Some(format!("Steam AppID {appid}")),
        url: appid_to_rss_url(appid),
      });
    }
  }

  if args.verify {
    let progress = ProgressBar::new(potential_feeds.len().try_into()?)
      .with_style(ProgressStyle::with_template("Verifying {pos}/{len} {bar}")?);

    for potential_feed in potential_feeds {
      let potential_feed = if [FeedOption::AppID, FeedOption::Url]
        .contains(&potential_feed.option)
      {
        let response = ureq_agent.get(&potential_feed.url).call()?;
        sleep(timeout);
        if response.content_type() == "text/xml" {
          let body = response.into_string()?;
          let title_start = body.find("<title>").unwrap() + 7;
          let title_end = body.find("</title>").unwrap();
          Feed {
            text: Some(body[title_start..title_end].to_string()),
            ..potential_feed
          }
        } else {
          continue;
        }
      } else {
        continue;
      };

      feeds_to_output.push(potential_feed);
      progress.inc(1);
    }
  } else {
    feeds_to_output = potential_feeds;
  }

  let mut opml_document = opml::OPML {
    head: None,
    ..Default::default()
  };

  for feed in feeds_to_output {
    if args.opml {
      opml_document
        .add_feed(&feed.text.unwrap_or_else(|| feed.url.clone()), &feed.url);
    } else {
      println!("{}", feed.url);
    }
  }

  if args.opml {
    println!("{}", opml_document.to_string()?);
  }

  Ok(())
}

/// Creates a Steam RSS URL from a given AppID.
fn appid_to_rss_url<D: std::fmt::Display>(appid: D) -> String {
  format!("https://steamcommunity.com/games/{appid}/rss/")
}
