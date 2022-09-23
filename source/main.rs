// Copyright (C) 2022 Bauke <me@bauke.xyz>
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
// details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! # Steam RSS
//!
//! > **Get RSS feeds for Steam games.**

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use std::{thread::sleep, time::Duration};

use {
  clap::Parser,
  color_eyre::{install, Result},
  regex::Regex,
  serde::Deserialize,
  serde_json::Value,
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

  /// A person's steamcommunity.com ID or full URL, can be used multiple times.
  #[clap(long)]
  pub user: Vec<String>,
}

/// A simple feed struct.
#[derive(Debug)]
pub struct Feed {
  /// A potential alternate friendly URL, see [`SteamApp::friendly_url`] for an
  /// explanation.
  pub friendly_url: Option<String>,

  /// The text to use for the feed in the OPML output.
  pub text: Option<String>,

  /// The URL of the feed.
  pub url: String,
}

/// A small representation of a Steam game that is parsed from JSON.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamApp {
  /// The AppID of the game.
  pub appid: usize,

  /// The name of the game.
  pub name: String,

  /// A friendly URL name of the game, some feeds will use this instead of their
  /// AppID for their RSS feed.
  ///
  /// For example, [Portal's feed](https://steamcommunity.com/games/Portal/rss)
  /// uses `Portal`, instead of
  /// [its AppID 400](https://steamcommunity.com/games/400/rss).
  ///
  /// Some games may also have a friendly URL different from their AppID but
  /// don't use it for their feed. Steam is weird.
  #[serde(rename = "friendlyURL")]
  pub friendly_url: Value,
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
  let user_json_regex = Regex::new(r"var rgGames = (?P<json>\[.+\]);\s+var")?;
  let user_id_regex = Regex::new(r"(i?)^\w+$")?;
  let user_url_regex =
    Regex::new(r"(?i)https?://steamcommunity.com/id/(?P<userid>\w+)")?;

  for appid in args.appid {
    potential_feeds.push(Feed {
      friendly_url: None,
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
        friendly_url: None,
        text: Some(format!("Steam AppID {appid}")),
        url: appid_to_rss_url(appid),
      });
    }
  }

  for user in args.user {
    let user_url = if user_id_regex.is_match(&user) {
      userid_to_games_url(user)
    } else if let Some(user) = user_url_regex
      .captures(&user)
      .and_then(|captures| captures.name("userid"))
    {
      userid_to_games_url(user.as_str())
    } else {
      continue;
    };

    let body = ureq_agent.get(&user_url).call()?.into_string()?;
    sleep(timeout);

    let games_json = user_json_regex
      .captures(&body)
      .and_then(|captures| captures.name("json"))
      .map(|json| json.as_str());
    if let Some(games_json) = games_json {
      let games = serde_json::from_str::<Vec<SteamApp>>(games_json)?;
      for game in games {
        let friendly_url = if game.friendly_url.is_string() {
          Some(appid_to_rss_url(game.friendly_url.as_str().unwrap()))
        } else {
          None
        };

        potential_feeds.push(Feed {
          friendly_url,
          text: Some(game.name),
          url: appid_to_rss_url(game.appid),
        });
      }
    } else {
      eprintln!("Couldn't scan games from: {user_url}");
      eprintln!(
        "Make sure \"Game Details\" in Privacy Settings is set to Public."
      );
      continue;
    }
  }

  if args.verify {
    let verify_feed = |url: &str| -> Result<_> {
      let response = ureq_agent.get(url).call()?;
      sleep(timeout);
      Ok((
        response.content_type() == "text/xml",
        response.into_string()?,
      ))
    };

    for mut potential_feed in potential_feeds {
      let (mut is_valid_feed, mut body) = verify_feed(&potential_feed.url)?;

      // If the potential URL doesn't return `text/xml`, try the friendly URL
      // if one exists.
      if !is_valid_feed && potential_feed.friendly_url.is_some() {
        let friendly_url = potential_feed.friendly_url.as_deref().unwrap();
        (is_valid_feed, body) = verify_feed(friendly_url)?;
        if is_valid_feed {
          potential_feed.url = friendly_url.to_string();
        }
      }

      let verified_feed = if is_valid_feed {
        let title_start = body.find("<title>").unwrap() + 7;
        let title_end = body.find("</title>").unwrap();
        Feed {
          text: Some(body[title_start..title_end].to_string()),
          ..potential_feed
        }
      } else {
        continue;
      };

      feeds_to_output.push(verified_feed);
    }
  } else {
    feeds_to_output.append(&mut potential_feeds);
  }

  let mut opml_document = opml::OPML {
    head: None,
    ..Default::default()
  };

  if feeds_to_output.is_empty() {
    eprintln!("No feeds found.");
    return Ok(());
  }

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

/// Creates a user's Steam Games URL from a given User ID.
fn userid_to_games_url<D: std::fmt::Display>(userid: D) -> String {
  format!("https://steamcommunity.com/id/{userid}/games/?tab=all")
}
