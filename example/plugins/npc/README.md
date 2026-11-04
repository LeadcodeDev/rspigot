# npc

An example plugin: it places a villager named "Guide" next to the world spawn and opens a menu when a player right-clicks it.

The villager has its name shown, no AI, no sound and cannot be hurt. The plugin removes guides left by an earlier run before it places a new one, and the guide stays after a server restart.

The menu holds three items: a book that sends a message, a compass that teleports the player to the spawn, and a barrier that closes the menu.

Start the server and the plugin as described in [`welcome`](../welcome/README.md), using `cargo run -p npc` for the plugin. Join and right-click the villager two blocks east of the spawn.

Only the main hand opens the menu: the client sends a second interaction for the off hand, which the plugin ignores.
