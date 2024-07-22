# treehouse content

The treehouse is built on its own custom data format called `.tree`.

I'll spare the explanation, because it's an _extremely_ simplistic format, but know that it nests:

- TOML metadata
- Djot markup

inside of itself.

TOML metadata is defined by `Attributes` from the `treehouse` crate. 

Djot on the other hand has support for generic attributes.
Keys are not documented (read the generator if you want to know about them,) but generally `:`-prefixed keys are
reserved for the treehouse's own purposes.

You probably don't want to read the content if you're not me.
Browse the treehouse yourself, experience it fully, and then when you're _sure_ you don't want to get spoiled, come back
here and look for weird stuff.
