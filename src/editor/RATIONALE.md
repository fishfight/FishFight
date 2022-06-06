TODO: Delete this file before merging!

- I changed the type of selected_tool from an option of type ID to an enum, because the available
tools are known at compiletime.
- The `actions` module file was too crowded, so I split all actions into their own files in `actions/`.
I think it's much more organized this way.
- Actions have been stripped from the `Action` suffix, as the parent module already tells us that
they are actions: `editor::actions::CreateLayerAction`. So instead of importing all the actions from
the editor and directly calling them with their names, I just import the actions module and refer to
them as `actions::CreateLayer` for instance. I think this is less redundant and also eliminates the
need to import all used actions, as their names are shorter.
- First, I tried to make the `ui` method in the editor state be immutable, sending *all* actions
through `UiAction`s being returned from the function. After a lot of work, I decided against making
it immutable. The reasoning behind this change is that passing all changes through the `UiAction`
enum is tiring and mechanical work that doesn't always have benefits, as there are some changes
that are *not* kept track of, such as opening windows, closing them, changing data within these
windows, etc. As of now, changes that do not require keeping track of are performed immediately,
while actions that do go through the history are sent through `apply_action` immediately.