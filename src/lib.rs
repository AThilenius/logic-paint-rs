use std::mem::forget;

use dom::{DomIntervalHooks, ElementEventHooks};
use miniz_oxide::inflate::decompress_to_vec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement};

use crate::substrate::deserialize_ic;

mod brush;
mod dom;
mod substrate;
mod utils;
mod v2;
mod viewport;
mod wgl2;

// Another new idea:

// Terms/concepts:
//  - Blueprint: Serialized cell chunks which include modules who's root resides in that chunk, as
//    well data blobs referenced by modules (ex. RAM data or JS source). Cells are packed into dense
//    lists of u32: [c_x: u8, c_y: u8, flags: u16]. Modules are bincode serialized. Blob data is
//    serialized in any format. All blueprints are stored as binary data that is gziped, then base64
//    UTF-8 encoded, with the preamble "LPBPV1:".
//  - Buffer: Unpacked Blueprint, in UPC format with an optional undo stack.
//    - Chunks (for cells) are blittable to the GPU and can be quickly serialized to Blueprints.
//      Modules are custom rendered, thus will never be blittable.
//    - Supports beginning, canceling, and committing a mutation transaction:
//      - Start of a transaction conceptually marks the 'before state' for an undo Blueprint.
//      - Mutating during a transaction is done by cloning the effected Chunk, and mutating the
//        clone in-place.
//        - A chunk is considered mutated when any bit in the UPC cells changes, or a bit in the
//          serialized module data changes (again, does not include the contents of blob refs).
//      - Cancelling a transaction effectively throws away all cloned chunks / modules.
//      - Committing first takes a snapshot of mutated chunks and modules in their starting state
//        (if the undo stack is enabled) before replacing the base chunks with the mutated ones.
//    - Keeps a generation counter (usize) for each chunk, which allows rendering code to quickly
//      check for mutation. The counter is rolled back when a transaction is canceled.
//    - Supports undo by keeping a stack of Blueprints, each representing a subset of buffer chunks
//      and modules from BEFORE a change was made. Overwrites each chunk in the undo frame.
//  - UPC format: Universal Packed Cell format stores each cell as a bit packed [u8; 4], ready for
//    direct blitting to a GPU RGBu8 texture. Stored as [u8; 4] instead of u32 for endian
//    agnosticism. Does not encode ActiveMask data. Modules are rendered separately from cells,
//    allowing each module to render itself differently.
//  - ActiveMask: a mask over a specific buffer to activate atoms, and/or highlight cells. Like
//    Buffer, it's stored in chunks that are blittable directly to the GPU. They represent the
//    second texture sampled per-fragment while rendering cells. It is used for both editing and
//    simulation state presentation. ActiveMask does not itself contain any logic for drawing to the
//    mask.
//  - Module: An overlay (with N number of I/O pins) that sits "on" an integrated circuit. All
//    modules belong to exactly one chunk, and are keyed on their "root" location. This location
//    does not need to be coincident a pin however, nor does a module need to have any pins at all
//    (ex. a label). Pins are not stored themselves, but are provided by the specific module.
//    However, the UPC format encodes pin presence, and that must always line up one-for-one with
//    the module provided pins; pin generation must be deterministic and idempotent.
//  - Range: An abstract "selection" of cell / modules which can be applied to a Buffer to get or
//    set a range of cells at once.
//  - AST: An analogy to an abstract syntax tree; stores a "compiled" buffer without execution
//    state; stores atoms, traces and gates. Used by both the execution engine (along with an
//    ExecutionState) and for presentation when updating an ActiveMask from an AST and
//    ExecutionState. ASTs are invalidated when a buffer changes.
//  - ExecutionState: State associated with an execution of a specific AST (notable gate state).
//    Invalidated when the corresponding AST is flushed.
//  - RenderContext: Stores all state associated with painting Buffers and ActiveMasks to the
//    screen. Does not however own the Camera, which is owned by the Context object. Includes GL
//    context, GL render target, shader programs, VBOs, VAOs, and textures associated with a
//    BufferChunk or ActiveMaskChunk. Chuck dirty tracking (re-upload to GPU is done with the
//    generation flags on chunks).
//  - Painter: Paints on a Buffer; the primary way a user draws things. Painter can selectively be
//    fed input events to enable or disable it.
//  - Context: The outer most object that owns the memory of everything visible in a given viewport,
//    for a given user session; owns the primary buffer, several ActiveMasks, any user edit state, a
//    compiled AST and ExecutionState (if executing) as well as the RenderContext, Camera and
//    Painter.

// Idk about the rest of this yet...

//  - Session: Stores all Buffers, state and ?compiler results? associated with a single editing
//    session. These include:
//    - Primary Buffer: The main buffer drawn / simulated. This maintains its own undo stack.
//    - Register Buffers: Any user assigned registers (`Ctrl+R <key>` or `"<key>`)
//    - Clipboard Register: The + register.
//  -
//  - IR: Intermediate Representation of a compiled Blueprint. This includes Atoms, Traces (each
//    trace is a vec of Atoms) and Gates.
//  - VM: Virtual machine for running IR. Manages simulation context (trace and gate state), I/O
//    module interop, and simulation stepping.

// Buffer - The non-simulation part of an IC today; defines cell layout, modules and any
//          annotations. It's the part that needs to be serialized. Buffers can be stored in
//          registers and simulated.
// LPIR - Logic-paint Intermediate Representation; anything compiled from a source Buffer like
//        traces and gates. Serializable, but probably always just JIT it.
// LPVM - Simulates LPIR and LPState.
// Simulation -
// IC - A compiled and ready to simulate Blueprint. Aka a Blueprint "instance". Only one exists.
// Active Buffer - The Blueprint that represents the IC (but not it's simulation state).
// Register - A place (named or otherwise) where a Blueprint can be stored.
// Insert-mode - Direct cell mutation on the Active Buffer.
// Visual-mode - Allows selecting cells, yanking, pasting and so on.
// Pasting-mode - An offshoot of insert mode, where a ghost of an IC follows the cursor and the user
//                can paste the blueprint as many times as they want.

// VIM stuff:

// Viewport and IC are exposed to JS. This is used in a few different ways:
//   - VIM-like scripts that the editor can execute. Ex a script for
//     generating a hardware ROM from a lookup table. Impl could be:
//     - Get cursor-follow register
//     - Programmatically place cells in IC
//     - User can place generated cells in main buffer
//   -

// Keybinds are modeled heavily after VIM
//   - Use cursor (crosshair and cell) to communicate insert vs visual-mode respectively. Can also
//     use grab and grabbing for moving a selection.
//   - Register access is done with `"<key>` like VIM.
//   - The `"=` register is the expression register. Like VIM it can be pasted with `Ctrl+r =` as
//     well.
//   - All scripts are JS (TS?), and have access to the current buffer's context. Use Monaco for
//     this.
// I/O Modules are all defined in Rust, but one of those modules is JavascriptIo
//   - JavascriptIo deferrers to JS callbacks for edge triggers and a few misc events like animation
//     frame.
//   - This needs to be as fast as possible, marshaling costs will be significant here.
// Base64 is always used for serialization, with a header for version info: `LPV1`.
// An entire IC can be yanked into a register, or a selection of it.
//   - When cells are "pasted", they follow the mouse until placed.
//   - When cells are "yanked", the point under the mouse is used as an anchor.
// To start with, selection is done with `v` and it's just rectangular.
//   - Shift to add to selection
//   - Ctrl to remove from selection
// For rendering
//   - Redefine "active" to be "selected", and how that is rendered is defined in uniforms. Then it
//     can be reused for things like selection highlighting, errors, and so on.
//   - Add one more bit for 'entire cell is selected' as is the case for visual-mode.

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    // let viewport = v2::Viewport::from_canvas(canvas.clone());

    // let dom_interval_hooks = result_or_log_and_return!(DomIntervalHooks::new(viewport.clone()));
    // let element_event_hooks = result_or_log_and_return!(ElementEventHooks::new(
    //     canvas.dyn_into::<HtmlElement>().unwrap(),
    //     viewport.clone()
    // ));

    // forget(dom_interval_hooks);
    // forget(element_event_hooks);

    // let substrate_viewport = result_or_log_and_return!(Viewport::from_canvas(canvas.clone()));

    // // Try to load from local storage
    // if let Ok(Some(compressed_b64)) = web_sys::window()
    //     .unwrap()
    //     .local_storage()
    //     .unwrap()
    //     .unwrap()
    //     .get_item(&"logic-paint-ic")
    // {
    //     if let Ok(compressed_bytes) = base64::decode(compressed_b64) {
    //         if let Ok(bytes) = decompress_to_vec(&compressed_bytes) {
    //             let ic = deserialize_ic(&bytes);
    //             substrate_viewport.borrow_mut().set_ic(ic);
    //         }
    //     }
    // }

    // let dom_interval_hooks =
    //     result_or_log_and_return!(DomIntervalHooks::new(substrate_viewport.clone()));
    // let element_event_hooks = result_or_log_and_return!(ElementEventHooks::new(
    //     canvas.dyn_into::<HtmlElement>().unwrap(),
    //     substrate_viewport.clone()
    // ));

    // forget(dom_interval_hooks);
    // forget(element_event_hooks);
}