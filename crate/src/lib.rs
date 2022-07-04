use wasm_bindgen::prelude::*;

mod blueprint;
mod brush;
mod buffer;
mod buffer_mask;
mod compiler;
mod coords;
mod dom;
mod execution_context;
mod logic_paint;
mod modules;
mod substrate;
mod upc;
mod utils;
mod viewport;
mod wgl2;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Simple module handling:
// - Single lp-modules.json file
// - Modules are managed almost entirely in LogicPaint, the only thing that VSCode does is inform
//   LP when the file changes, and handles events from LP like 'open module main-clk'. The modules
//   file itself stores the module's root, so you can't get orphaned modules. Copy/paste will be
//   handled in LP itself (module copying).
// - Module objects are linked with a 'module ID' (UUID). Can use those to jump-to line location
//   when LP asks to open a module definition too.
// - I like this more. Simpler.
// - For now, placing a module can be done by manually editing the JSON file.
//
// - Need a cursor-follow buffer accessible from JS, for pasting into. It is None by default.
//   Ideally it should be serialized such that I can move my mouse from one window split to another
//   and it follows.
// - There are at least 2 modes...
//   - Visual (default starting mode, accessed from any other mode with ESC)
//     - Denoted by the cell-cursor (Excel style)
//     - Only mode where module anchors are visible
//     - Same selection keybinds as Excel. Clicking/Dragging selected a range. Holding Shift adds to
//       the selection. Holding Ctrl removes from the selection.
//     - Hovering a trace highlights the conductive path
//     - Double-clicking a trace selects the conductive path cells
//     - Modules can be poked. Dragging a module's anchor selects it for moving (moves it into the
//       cursor-follow buffer). Pasted on mouse-up. Not sure I love that.
//     - VSCode::OnCopy copies the selected cells and modules, with the root being what ever cell
//       was last under the mouse at that time.
//     - VSCode::OnPaste pastes into a 'cursor follow' buffer, next mouse click commits it to active
//   - Paint mode (via hitting any brush keybind)
//     - Denoted with a crosshair-cursor while over cells.
//     - All paint modes: LMB primary, RMB || Shift+LMB secondary, Ctrl+[...] to erase.
//     - Paint Metallic (keybind: F):
//       - LMB: paint, RMB || Shift+LMB: Via, Ctrl+... to remove
//     - Paint Si (keybind: D):
//       - LMB: paint N, RMB || Shift+LMB paint P, Ctrl+LMB to erase any type
//     - (Stretch goal) Paint conductive path (keybind: R);
//       - LMB: paint metal when able, drop down to Si to bridge metal when able.
//       - RMB: paint Si when able, step up to metal to bridge Si when able.
//   - In all modes
//     - MiddleMouse || Space+LMB pans the camera
//     - Scrollwheel zooms in/out
//     - Lower H returns to home at same zoom, upper H returns to home at default zoom

// Micro-tasks
// - Use a + cursor when interacting with cells (use 'cell' cursor in visual mode)
// - Figure out how to draw under module HTML
// - Rendering bug between chunks

// =================================================================================================

// Terms/concepts:
//  X Blueprint: Serialized cell chunks which include modules who's root resides in that chunk, as
//    well data blobs referenced by modules (ex. RAM data or JS source). Cells are packed into dense
//    lists of u32: [c_x: u8, c_y: u8, flags: u16]. Modules are bincode serialized. Blob data is
//    serialized in any format. All blueprints are stored as binary data that is gziped, then base64
//    UTF-8 encoded, with the preamble "LPBPV1:".
//  x Buffer: Unpacked Blueprint, in UPC format with an optional undo stack.
//    x Chunks (for cells) are blittable to the GPU and can be quickly serialized to Blueprints.
//      Modules are custom rendered, thus will never be blittable.
//    x Supports beginning, canceling, and committing a mutation transaction:
//      x Start of a transaction conceptually marks the 'before state' for an undo Blueprint.
//      x Mutating during a transaction is done by cloning the effected Chunk, and mutating the
//        clone in-place.
//        x A chunk is considered mutated when any bit in the UPC cells changes, or a bit in the
//          serialized module data changes (again, does not include the contents of blob refs).
//      x Cancelling a transaction effectively throws away all cloned chunks / modules.
//      x Committing first takes a snapshot of mutated chunks and modules in their starting state
//        (if the undo stack is enabled) before replacing the base chunks with the mutated ones.
//    x Supports undo by keeping a stack of Blueprints, each representing a subset of buffer chunks
//      and modules from BEFORE a change was made. Overwrites each chunk in the undo frame.
//  x UPC format: Universal Packed Cell format stores each cell as a bit packed [u8; 4], ready for
//    direct blitting to a GPU RGBu8 texture. Stored as [u8; 4] instead of u32 for endian
//    agnosticism. Does not encode BufferMask data. Modules are rendered separately from cells,
//    allowing each module to render itself differently.
//  X BufferMask: a mask over a specific buffer to activate atoms, and/or highlight cells. Like
//    Buffer, it's stored in chunks that are blittable directly to the GPU. They represent the
//    second texture sampled per-fragment while rendering cells. It is used for both editing and
//    simulation state presentation. BufferMask does not itself contain any logic for drawing to the
//    mask.
//  X Module: An overlay (with N number of I/O pins) that sits "on" an integrated circuit. All
//    modules belong to exactly one chunk, and are keyed on their "root" location. This location
//    does not need to be coincident a pin however, nor does a module need to have any pins at all
//    (ex. a label). Pins are not stored themselves, but are provided by the specific module.
//    However, the UPC format encodes pin presence, and that must always line up one-for-one with
//    the module provided pins; pin generation must be deterministic and idempotent. Finally,
//    modules are not frustum culled (they are always rendered)
//  x Range: An abstract "selection" of cell / modules which can be applied to a Buffer to get or
//    set a range of cells at once.
//  X CompilerResults: An analogy to an abstract syntax tree; stores a "compiled" buffer without
//    execution state. Stores atoms, traces and gates. Used by both the execution engine (along with
//    an ExecutionState) and for presentation when updating an BufferMask from a CompilerResults and
//    ExecutionState. CompilerResults are invalidated when a buffer changes.
//  X ExecutionState: State associated with an execution of a specific AST (notable gate state).
//    Invalidated when the corresponding AST is flushed.
//  X RenderContext: Stores all state associated with painting Buffers and BufferMasks to the
//    screen. Does not however own the Camera, which is owned by the Context object. Includes GL
//    context, GL render target, shader programs, VBOs, VAOs, and textures associated with a
//    BufferChunk or BufferMaskChunk. Chuck dirty tracking (re-upload to GPU) is done with the
//    generation counter on chunks.
//  X Painter: Paints on a Buffer; the primary way a user draws things. Painter can selectively be
//    fed input events to enable or disable it.
//  X LogicPaintContext: The outer most object that owns the memory of everything visible in a given
//    viewport, for a given user session; owns the primary buffer, several BufferMasks, any user
//    edit state, a compiled AST and ExecutionState (if executing) as well as the RenderContext,
//    Camera and Painter.
