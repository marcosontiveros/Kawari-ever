# Infrastructure

In the event of a project closure, forking or other event this document should be used to keep track of the state of our infrastructure and access.

## Code repository

The project source code is hosted on [Codeberg](https://codeberg.org/redstrate/Kawari), and currently the only user with write privileges is [@redstrate](https://codeberg.org/redstrate).

An official mirror is hosted on [code.ryne.moe](https://code.ryne.moe/redstrate/kawari) in the event the canonical one is inaccessible. This is only meant as a read-only mirror and is not set up for contributions.

## Continuous Integration

The project uses Forgejo Actions for CI, and only uses runners paid by redstrate out-of-pocket.

## Chat

The Matrix chat is maintained by @redstrate:pyra.sh, who is also the room's creator. The room is currently low traffic enough that moderation is done solely by @redstrate. Matrix is decentralized so even if the domain pyra.sh is offline, the room will still function for everyone else.

The room is invite-only, with access gated behind a simple bot under the ID @bot:pyra.sh. The reason is two-fold:
* It should keep out most dumb spammers who enter public rooms.
* If my server shuts down for whatever reason the room is "locked" from the outside. It would still be unmoderated, but it's at least a good start.

The source code for the bot is located [here](https://codeberg.org/redstrate/moderation-node).
