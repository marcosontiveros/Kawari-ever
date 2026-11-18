# Event API

We currently support scripting CustomTalk and Quest events with Lua.

## Globals

### `EVENT_ID`

Integer representation of the event ID.

### `BASE_ID`

Only set for Event NPCs and Event Objects, this is their base ID.

### `GAME_DATA`

Access to the `GameData` struct.

### `CONDITION`

Can be set by the script to override the condition set when the event is running. If not set, defaults to `Condition::OccupiedInQuestEvent`.

### `DISABLE_EVENT_POS_ROLLBACK`

Can be set by the script to disable event position rollback. If not set, defaults to `false`.

For certain events, you also have access to some globals set by the client.
Any available ones are printed to the world server log when starting said event.

## Functions

### `onEnterTerritory(player)`

Called when the client enters the territory, only used by certain event handlers.

* `player` is the `LuaPlayer` which represents the player.

### `onEnterTrigger(player, arg)`

Called when the client enters a walk-in/walk-outside event trigger.

* `player` is the `LuaPlayer` which represents the player.
* `arg` is an arbitrary value sent by the client.

### `onTalk(target, player)`

This is called when a player begins talking with the actor.

* `target` is the ID of the actor being spoken to.
* `player` is the `LuaPlayer` which represents the player.

### `onYield(scene, id, results, player)`

This is called when a scene yields execution.

* `scene` is the scene number.
* `id` is the arbitrary yield ID.
* `results` is an array of results returned by the client.
* `player` is the `LuaPlayer` which represents the player.

### `onReturn(scene, results, player)`

This is called when a scene returns.

* `scene` is the scene number.
* `results` is an array of results returned by the client.
* `player` is the `LuaPlayer` which represents the player.

### `onNestedEventFinished(type, results, player)`

This is called when a nested event finishes.

* `type` is `HandlerType` of the event that finished.
* `results` is an array of results returned by the client.
* `player` is the `LuaPlayer` which represents the player.


