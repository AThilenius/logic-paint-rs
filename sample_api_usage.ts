import { Viewport, Session } from 'logic-paint';

// Load data from firestore...
const dataFromFirestore = '...';
const session = Session.from_base_64(dataFromFirestore);

const canvas = document.getElementById('id') as HTMLCanvasElement;

// Hooks all the input events it could need automatically.
const viewport = Viewport.from_existing_canvas(canvas);
viewport.set_editing_enabled(false);
viewport.set_session(session);

// Control of simulation is done through the session object.
const simulation = session.beginSimulation();

// When JS starts the sim, then the viewport is essentially read-only?

// Dual-mode like a code editor:
// - Editing: user can mutate buffers.
// - Running: user cannot edit anything, and has run controls.

// Run modes:
// - Single step: User can single-step the sim loop with a key.
// - Free run: Loops is run up to MAX_UPS/FRAMERATE each frame.
// - Record: Record
