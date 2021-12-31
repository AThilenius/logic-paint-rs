import { LogicPaint, Mode } from 'logic-paint';

// Load data from Firestore or wherever.
const strFromFirestore = '...';

// Canvas element will always be passed into Rust.
const canvas = document.getElementById('id') as HTMLCanvasElement;

// Main state-machine for a Logic Paint instance.
const logicPaint = new LogicPaint(canvas);

// A `Session` contains a 'user session', meaning all data that a user would
// want when rehydrating an editor.
logicPaint.set_session_from_base_64(strFromFirestore);

// For example, if you wanted to put it into readonly (view) mode.
logicPaint.set_read_only(true);
logicPaint.set_sim_config({
  mode: 'limit-ticks-per-frame',
  ticks: 1,
});
logicPaint.set_mode(Mode.Simulating);

// Run modes:
// - Single step: User can single-step the sim loop with a key.
// - Free run: Loops is run up to MAX_UPS/FRAMERATE each frame.
// - Record: Record
