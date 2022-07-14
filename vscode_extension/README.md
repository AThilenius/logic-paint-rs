# Logic Paint

## Files

Logic Paint substrate blueprints are defined in (JSON encoded) files that end
with the `.lpbp` (Logic Paint BluePrint) extension.

Module configuration is done in raw JSON, in a file with the same name as the
blueprint file, but ends in `.lpm.json` (Logic Paint Modules).

## Standard Controls

The camera can be panned at any time (including while actively painting) by
holding down the `Space` key. It can be zoomed at any time with the scroll
wheel.

Standard controls like `Ctrl+Z` and `Ctrl+S` to undo and save should work just
fine.

## Modal Editing

Logic Paint is modal, the active mode is displayed in the upper left corner of
the editor window. It has 4 primary modes:

- `Visual` (the starting mode) lets you select, copy, paste and delete cells.
- `Silicon` paints (or removes) both N and P type silicon doping onto the
  substrate.
- `Metal` paints (or removes) a metal layer and via placements.
- `Execution` compiles logic and prepares to execute it.

## Visual Mode (`ESC`)

Visual mode is accessed by clicking `ESC` from any other mode. While in visual
mode you can perform the following actions:

- **`LMB`** Create a selection with `LMB` and dragging. Clear the selection with
  `ESC` or `RMB`.
- **`Y`** Copy ("yank") the current selection into the mouse-follow buffer. Use
  `LMB` to paste the mouse-follow buffer.
- **`D`** Copy the current selection into the mouse-follow buffer _and_ delete
  the selection.
- **`Shift+D`** Delete the selection _without_ copying it into the mouse-follow
- **`S+[0-9]`** With a selection _or_ something currently in the mouse-follow
  buffer, you can hold `S` and click a number to 'store' that into a numbered
  register.
- **`[0-9]`** Once something is stored in a numbered register, you can load it
  into the mouse-follow buffer by clicking the number (without holding `S`).
- **`Shift+S+8` and `Shift+8`** Like VIM, there is a special register for
  copying/pasting into the system clipboard, the `*` register. This register
  will behave like the numbered registers, apart from the fact that it will
  always contain the system-clipboard value.
- **`R`** while the mouse-follow buffer is active to rotate it.

### Visual Mouse-follow Buffer

There is a special buffer that is used for copy-paste and rotation operations,
the "mouse-follow buffer". It is filled when:

- A selection is yanked with `Y`, or deleted with `D`
- A named register is selected using `[0-9]` or `*`

While the mouse-follow buffer is active, it will show a preview of the **results
of placing the buffer onto the main blueprint**, not the contents of the buffer
itself. Meaning, the mouse-follow buffer can contain invalid connections (ie
cells that claim to connect with a neighbor who doesn't connect back) and these
broken connections will be 'stitched together' on-paste, when possible. The
mouse-buffer is ephemeral and can be cleared at any time by clicking `ESC` or
`RMB`. Additionally, clicking `R` will rotate the mouse-follow buffer 90 degrees
clockwise.

## Silicon Mode (`Q`)

Silicon mode is accessed with the `Q` key. It is used for painting and erasing
silicon doping. While in Silicon mode you can paint N and P type silicon, and
form gates by dragging N-type on top of a P-type trace, or vice versa.

- **`LMB`** Paint a line of N-type silicon
- **`RMB` (Or `Shift+LMB`)** Paint a line of P-type silicon
- **`Ctrl+LMB`** Erase silicon of either type

## Metal Mode (`W`)

Metal mode is accessed with the `W` key. It is used to connect Silicon together.
It lives "above" the silicon and does not touch the silicon unless you place a
via to connect them.

- **`LMB`** Paint a line of metal.
- **`RMB` (Or `Shift+LMB`)** Places a Via. Can only be used on a cell that has
  both metal and silicon and isn't a transistor.
- **`Ctrl+LMB`** Erase metal and vias.

## Execution Mode (`E`)

Execution mode is accessed with the `E` key. It used to 'simulate' the
substrate. It allows for both single-stepping as well as continuous running
modes. Right now the only run mode is one fundamental clock per frame, but more
will be added later for faster running.

- **`R`** Enters run-mode. This will execute one fundamental clock per frame.
- **`C`** Pauses run mode (if running) and executes a single fundamental clock.
- **`P`** Pauses run mode.
- **`T`** Executes a single simulation 'tick'. This is mostly for debugging
  Logic Paint itself, as ticks have very little parallel with propagation delay.

## Modules

Modules are configured in JSON (see above for file naming). The JSON file is an
array of module parameters, including where they live in the blueprint. Saving
this file will auto-update the modules in the blueprint if it's open.
