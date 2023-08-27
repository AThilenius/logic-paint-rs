Dev:

In js-sys:
npm run wasm
npm run dev

Re-run `npm run wasm` to rebuild Rust `core`.

Project Structure:

## `core`

- Rust crate
- Both the 'host' used from TS on the website, and the crate plugins add to
  their `Cargo.toml` to author a plugin.
- Handles 'taking in' an HTML Canvas, and setting everything up, including
  the OpenGL rendering context and egui.
- Creates SharedArrayBuffers for
  - Editor state (selection, camera, registers)
  - Buffers (active, mouse-follow, ephemeral)
  - Execution context (sim-state, buffer-mask(s), compiler-results)
- Handles loading and hosting plugin WASM modules
  - Loads WASM module from `UInt8Array` that includes the exposed wasm-bindgen
    bindings from the linked `logic-paint` crate.
  - Shares all necessary `SharedArrayBuffers` with the module for low-overhead
    calling.
  - Calls exposed plugin hooks at the right time.
  - Allows plugins to "prevent" event bubbling, in essence "consuming" input.
- Handles I/O serialization and passing to plugins
- While it doesnt' contain the code to handle user-input for drawing Si/metal,
  it does host the code in `brush.rs`, which is likely to be useful to others.

## `core-plugin`

- Rust crate
- Links `core` for the plugin exports
- Hosts all the standard editor and module functionality
- Hosts the various editor modes, Si, metal et. al, using paint code in core.

## `web`

- Either a react or svelte app (initially react) to host `logic-paint.com`
- Uses `core` and `core-plugin`

## POC

So what do I have for an early POC here?

- `core` crate
  - [x] Includes a template `egui` app
  - Exposes the plugin types, and `SharedArrayBuffers`
  - Loads the next crate as a demo plugin via an `include_bytes!` to the `.wasm`
- `core-plugin` crate
  - Does some hello-world rendering to the shared buffers
  - Renders some egui widgets
- `web` react site
  - [x] Uses `core` via calling exposed WASM functions

Overview:

- Rust code is exclusively rendering, it has to be fed data and I/O events
  manually. These include mouse input, keyboard, having blueprint updates,
  clipboard operations, camera controls, and so on. That'll keep the WASM
  teeny-tiny, which will allow me to use it directly for thumbnail previews.
- Might also be worth splitting out rendering from the primary logic, but
  statically compile them together. This will make headless exploration of
  blueprints much easier in the future. It might also make extension-based
  modules/tool easier too.
- Modules and ideally tools should be extension based, running in their own
  WASM modules. I might want to make this a stretch goal though.
- WASM gets wrapped in a "low-level" TS module which handles basic I/O and
  canvas creation / resizing. It's an imperative API, not tied to any
  rendering framework like Svelte/React.

Steps:

- Pull all I/O events out into serialized data structures (I think this is
  already done?)
- Remove Yew
- Split rendering out a bit more, maybe
- Wrap things in JS and get an early 'it renders' POC going

- Rust Crate
  - Handles all the stuff it does today, plus automatic DOM hooks
- JS Lib
  - Wraps the Rust Crate in a pure-JS library
- React lib
  - Wraps the JS lib in React components
- Site
  - The logic-paint.com site.

Rust Core Modifications

- Use egui for all UI. This is the big one.
- Handle directly binding the DOM for events like clipboard access.
- Stretch-goal: create a read-only mode
- Use atlas texture rendering to reduce draw calls
- Modules should be plugin based
  - Use Wasm Bindgen directly, like this:
    https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-wasm.html
  - Modules are registered by UUID, and can serialize what ever data they want
  - Performance test this
  - Just like there were before, they have a 'root' and N "pins" which contact
    the metal layer.
  - Might need to extend egui to handle the serialization layer. Should be
    possible, their docs for 'implementing a backend' were pretty
    straightforward. Capturing I/O might be a bit more tricky, but we'll see.

General serialization strategy:

- Blueprints contain
  - Chunk versions: (chunk_x, chunk_y, content_hash_xxh3)
    - Chunk content is stored RLN, as a Base64.
    - Content hash is made on the binary data before the Base64 encoding.
  - Modules: (cell_root_x, cell_root_y, module_hash_xxh3)
    - Module serialization is up to the module itself, any textual serializer
      is acceptable.
- All hashes are XXH3 (128-bit) from https://github.com/DoumanAsh/xxhash-rust

The JS lib needs to support

- Create a LogicPaintCanvas from an HTMLCanvas element. Hosts exactly one
  Blueprint. Multi-panel can be supported with multiple instances, synced in
  JS or via internet.
- Set it in preview (readonly) mode.
- Clearing of a Blueprint (resets the LogicPaintCanvas)
- Both overwrite and (stretch goal) additive setting of a Blueprint
  - In overwrite mode any chunk provided in the Blueprint fully overwrites the
    previous chunk.
  - Stretch goal: in additive mode, it works the same way pasting does, where
    only non-empty cells are added to each chunk
- Provide a hook for saving an update to the Blueprint

Site

- Use PocketBase and Svelte!
- Blueprints can be stored directly in PocketBase, as B64 strings
- Use Google and Github for logins

Plugin bindings

- All stages have access to all editor state, including:
  - Buffers (primary, mouse-follow, stashed)
  - Camera
  - Mode
- early_update
  - Called once per update loop, before input events are dispatched
- dispatch_event
  - Called once per event that was dispatched this frame
  - Events are primarily human input (keyboard/mouse) but are generic.
- late_update
  - Called once per update loop, after ALL input events are dispatched
- draw
  - Called before the core dispatches WebGL calls
  - Allows GUI draw commands to be serialized and sent over
  - Has access to basic substrate render context knobs, like drawing the
    mouse-follow lines, or changing the shader code
