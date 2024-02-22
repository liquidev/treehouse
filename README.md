# liquidex's treehouse

Welcome to the Construct.

If you haven't seen the treehouse yet, you [may wanna](https://liquidex.house). It's pretty darn cool.

Please note that this repository contains spoilers. So if you like exploring by yourself, you may wanna do that first before diving into the source code.

*Spoilers for what?*, you might ask.

…

You have been warned.

## Building

The following commands require [`just`](https://github.com/casey/just).
Running a development server requires [`cargo-watch`](https://github.com/watchexec/cargo-watch).

To serve the website on `http://localhost:8080` for development:

```sh
just
```

This will start a server on port 8080. You can change the port by using the variable `port`:

```sh
just port=8081
```

The website will reload itself automatically if you change any file in the repository.

## Contributing

If you found a typo, be my guest. Just note that some typos are intentional, please make sure you understand the full context of the sentence.

If you found a bug, by all means please submit a fix. (or at least an issue.)

Since this is my personal website, I don't accept outside contributions for new content. Because then it would no longer be *my* treehouse.

If you wish to create something similar to liquidex's treehouse, you probably want to use more mature software instead of my scrappy, opinionated piece of art. Check out [Logseq](https://logseq.com/) - it has static site generation built in, with a much more approachable UI, cross-device sync, and great customizability through community-made themes and plugins.
