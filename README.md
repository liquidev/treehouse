# liquidex's treehouse

Welcome to the Construct.

If you haven't seen the treehouse yet, you [may wanna](https://liquidex.house). It's pretty darn cool.

Please note that this repository contains spoilers. So if you like exploring by yourself, you may wanna do that first before diving into the source code.

*Spoilers for what?*, you might ask.

…

You have been warned.

## Building

To build the website:

```sh
cargo run -p treehouse regenerate
```

This will spit out a directory `target/site` containing the static pages. You're free to use any HTTP server you wish, but for development purposes treehouse includes one in the CLI:

```sh
cargo run -p treehouse regenerate --serve
```

This will fire up a server on port 8080. No way to change that, sorry. Edit the source code.

If you're developing, you may wanna use [`cargo-watch`](https://crates.io/crates/cargo-watch):

```sh
cargo watch -- cargo run -p treehouse regenerate --serve
```

The website will reload itself automatically if you change any file in the repository.

## Contributing

If you found a typo, be my guest. Just note that some typos are intentional, please make sure you understand the full context of the sentence.

If you found a bug, by all means please submit a fix. (or at least an issue.)

Since this is my personal website, I don't accept outside contributions for new content. Because then it would no longer be *my* treehouse.

If you wish to create something similar to liquidex's treehouse, you probably want to use more mature software instead of my scrappy, opinionated piece of art. Check out [Logseq](https://logseq.com/) - it has static site generation built in, with a much more approachable UI, cross-device sync, and great customizability through community-made themes and plugins.
