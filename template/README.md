# Templates

This directory houses Handlebars templates, which are mostly used for reusable bits of the house.

Files that are not prefixed with a `_` are generated into their own `.html` files.
All other files are only loaded into Handlebars for use by other templates (or the generator itself.)

In particular, `_tree.hbs` is used as the default page template. This can be changed by including a `%% template = "_whatever.hbs"` at the top of your .tree file.
