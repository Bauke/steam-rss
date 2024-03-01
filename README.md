# Steam ❤ RSS

> **Get RSS feeds for Steam games.**

## Features

* Get RSS feeds from a game's AppID or store page.
* Get RSS feeds for all games from a user profile.
* Verify potential feeds by checking if they return `text/xml`.
* Output feeds as an OPML file for easy importing.

## Installation

### Cargo

With a working [Rust and Cargo](https://www.rust-lang.org/learn/get-started) installation, you can install `steam-rss` from [Crates.io](https://crates.io/crates/steam-rss).

```
cargo install steam-rss
```

## Usage

```
USAGE:
    steam-rss [OPTIONS]

OPTIONS:
    -a, --appid <APPID>        A game's AppID, can be used multiple times
    -h, --help                 Print help information
        --opml                 Output the feeds as OPML
    -t, --timeout <TIMEOUT>    The time in milliseconds to sleep between HTTP requests [default:
                               250]
        --url <URL>            A game's store URL, can be used multiple times
        --user <USER>          A person's steamcommunity.com ID or full URL, can be used multiple
                               times
    -v, --verify               Verify potential feeds by downloading them and checking if they
                               return XML
    -V, --version              Print version information
```

## Feedback

Found a problem or want to request a new feature? Email [me@bauke.xyz](mailto:me@bauke.xyz) and I'll see what I can do for you.

## License

Distributed under the [AGPL-3.0-or-later](https://spdx.org/licenses/AGPL-3.0-or-later.html) license, see [LICENSE](https://github.com/Bauke/steam-rss/blob/main/LICENSE) for more information.
