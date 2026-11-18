# LuaPlayer

## Functions

### `play_scene(scene, flags, params)`

Starts `scene` given `flags` and a list of `params`.

### `finish_event(args)`

Forcefully finishes the current event, and either returns control back to the player or the event that nests this one.

* `args` is optional, but is an arbitrary number list that's then given to the parent event's `onNestedEventFinished`.

### `start_event(id, type, arg)`

Starts another event of `id`, usually paired with `EVENT_TYPE_NEST` as `type` for nesting.
