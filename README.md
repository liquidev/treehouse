# liquidex's treehouse

Welcome to the Construct.

If you haven't seen the treehouse yet, you [may wanna](https://liquidex.house). It's pretty darn cool.

Please note that this repository contains spoilers. So if you like exploring by yourself, you may wanna do that first before diving into the source code.

*Spoilers for what?*, you might ask.

…

You have been warned.

## Building

To serve the website on `http://localhost:8080`:

```sh
cargo run -p treehouse serve
```

This will start a server on port 8080. You can change the port by using `--port`, but note that you'll also have to override the website address. The treehouse hardcodes all URLs to point to its own address and therefore needs a base URL, provided with `$TREEHOUSE_SITE` or more permanently by setting `site` inside `treehouse.toml`:

```sh
TREEHOUSE_SITE="http://localhost:8081" cargo run -p treehouse serve --port 8081
```

If you're developing, you may wanna use [`cargo-watch`](https://crates.io/crates/cargo-watch):

```sh
cargo watch -- cargo run -p treehouse serve
```

The website will reload itself automatically if you change any file in the repository.

## Contributing

If you found a typo, be my guest. Just note that some typos are intentional, please make sure you understand the full context of the sentence.

If you found a bug, by all means please submit a fix. (or at least an issue.)

Since this is my personal website, I don't accept outside contributions for new content. Because then it would no longer be *my* treehouse.

If you wish to create something similar to liquidex's treehouse, you probably want to use more mature software instead of my scrappy, opinionated piece of art. Check out [Logseq](https://logseq.com/) - it has static site generation built in, with a much more approachable UI, cross-device sync, and great customizability through community-made themes and plugins.
