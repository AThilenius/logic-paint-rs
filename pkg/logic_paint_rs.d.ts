/* tslint:disable */
/* eslint-disable */
/**
 * Convert a legacy blueprint JSON file into a Buffer (which can then be saved into the latest
 * format). Does not support modules, only the substrate is loaded.
 * @param {string} json_str
 * @returns {Buffer}
 */
export function import_legacy_blueprint(json_str: string): Buffer;
export function main(): void;
export enum CellPart {
  Metal = 0,
  Si = 1,
  EcUpLeft = 2,
  EcDownRight = 3,
}
export type Metal = { type: "None" } | { type: "Trace"; data: { has_via: boolean; placement: Placement } };

export type Silicon = { type: "None" } | { type: "NP"; data: { is_n: boolean; placement: Placement } } | { type: "Mosfet"; data: { is_npn: boolean; is_horizontal: boolean; gate_placement: Placement; ec_placement: Placement } };

export class Atom {
  free(): void;
  coord: CellCoord;
  part: CellPart;
}
export class BoolState {
  free(): void;
/**
 * The key was just clicked this dispatch.
 */
  clicked: boolean;
/**
 * The key is being held down. Can be true when `clicked` is true.
 */
  down: boolean;
/**
 * The key was just released this dispatch.
 */
  released: boolean;
}
/**
 * Buffers are an infinite grid of cells, where each cell is 4 bytes. Things are split into
 * chunks, where each chunk stores a simple Vec<u8>, and Chunks are indexed by their chunk
 * coordinate on the infinite grid. Chunks with zero non-default cells take up no memory.
 *
 * This struct is cheap to clone, as chunks are Copy-On-Write thanks to `im` HashMap. Sockets
 * however are cloned in their entirety, because they are relatively small.
 */
export class Buffer {
  free(): void;
  constructor();
  /**
   * @param {CellCoord} cell_coord
   * @returns {UPC}
   */
  get_cell(cell_coord: CellCoord): UPC;
  /**
   * @param {CellCoord} cell_coord
   * @param {UPC} cell
   */
  set_cell(cell_coord: CellCoord, cell: UPC): void;
  /**
   * @param {Selection} selection
   * @param {CellCoord} anchor
   * @returns {Buffer}
   */
  clone_selection(selection: Selection, anchor: CellCoord): Buffer;
  /**
   * @param {CellCoord} cell_coord
   * @param {Buffer} buffer
   */
  paste_at(cell_coord: CellCoord, buffer: Buffer): void;
  /**
   * @returns {Buffer}
   */
  rotate_to_new(): Buffer;
  /**
   * @returns {Buffer}
   */
  mirror_to_new(): Buffer;
  fix_all_cells(): void;
  /**
   * @returns {number}
   */
  cell_count(): number;
  /**
   * @returns {string}
   */
  to_base64_string(): string;
  /**
   * @returns {Uint8Array}
   */
  to_bytes(): Uint8Array;
  /**
   * @param {string} base_64_string
   * @returns {Buffer}
   */
  static from_base64_string(base_64_string: string): Buffer;
  /**
   * @param {Uint8Array} bytes
   * @returns {Buffer}
   */
  static from_bytes(bytes: Uint8Array): Buffer;
  /**
   * @param {CellCoord} arg0
   * @param {CellCoord} arg1
   * @param {boolean} initial_impulse_vertical
   * @param {boolean} paint_n
   */
  draw_si(arg0: CellCoord, arg1: CellCoord, initial_impulse_vertical: boolean, paint_n: boolean): void;
  /**
   * @param {CellCoord} arg0
   * @param {CellCoord} arg1
   * @param {boolean} initial_impulse_vertical
   */
  draw_metal(arg0: CellCoord, arg1: CellCoord, initial_impulse_vertical: boolean): void;
  /**
   * @param {CellCoord} arg0
   * @param {CellCoord} arg1
   * @param {boolean} initial_impulse_vertical
   */
  clear_si(arg0: CellCoord, arg1: CellCoord, initial_impulse_vertical: boolean): void;
  /**
   * @param {CellCoord} arg0
   * @param {CellCoord} arg1
   * @param {boolean} initial_impulse_vertical
   */
  clear_metal(arg0: CellCoord, arg1: CellCoord, initial_impulse_vertical: boolean): void;
  /**
   * @param {CellCoord} cell_coord
   */
  draw_via(cell_coord: CellCoord): void;
  /**
   * @param {Selection} selection
   */
  clear_selection(selection: Selection): void;
  /**
   * @param {Selection} selection
   */
  clear_selection_border(selection: Selection): void;
  /**
   * @param {CellCoord | undefined} from
   * @param {CellCoord} to
   * @param {boolean} paint_n
   */
  draw_si_link(from: CellCoord | undefined, to: CellCoord, paint_n: boolean): void;
  /**
   * @param {CellCoord | undefined} from
   * @param {CellCoord} to
   */
  draw_metal_link(from: CellCoord | undefined, to: CellCoord): void;
  /**
   * @param {CellCoord} cell_coord
   */
  clear_cell_si(cell_coord: CellCoord): void;
  /**
   * @param {CellCoord} cell_coord
   */
  clear_cell_metal(cell_coord: CellCoord): void;
}
export class Camera {
  free(): void;
  /**
   * @param {Vec2} translation
   * @param {number} scale
   */
  constructor(translation: Vec2, scale: number);
  /**
   * Project a screen x,y point into the world. Z axis is ignored because I don't need it.
   * @param {Vec2} position
   * @returns {Vec2}
   */
  project_screen_point_to_world(position: Vec2): Vec2;
  /**
   * Project a screen point to a cell location. It's the caller's responsibility to ensure the
   * point is within the visible bounds of the window.
   * @param {Vec2} position
   * @returns {CellCoord}
   */
  project_screen_point_to_cell(position: Vec2): CellCoord;
  /**
   * @param {CellCoord} coord
   * @returns {Vec2}
   */
  project_cell_coord_to_screen_point(coord: CellCoord): Vec2;
  scale: number;
  size: Vec2;
  translation: Vec2;
}
export class CellCoord {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  0: IVec2;
}
export class CompilerResults {
  free(): void;
  /**
   * @param {Buffer} buffer
   */
  constructor(buffer: Buffer);
  /**
   * @param {Buffer} buffer
   * @param {Atom} edge_atom
   * @returns {(Atom)[]}
   */
  static get_trace_atoms(buffer: Buffer, edge_atom: Atom): (Atom)[];
}
/**
 * A 2x2 column major matrix.
 */
export class DMat2 {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m10
   * @param {number} m11
   */
  constructor(m00: number, m01: number, m10: number, m11: number);
  x_axis: DVec2;
  y_axis: DVec2;
}
/**
 * A 3x3 column major matrix.
 *
 * This 3x3 matrix type features convenience methods for creating and using linear and
 * affine transformations. If you are primarily dealing with 2D affine transformations the
 * [`DAffine2`](crate::DAffine2) type is much faster and more space efficient than
 * using a 3x3 matrix.
 *
 * Linear transformations including 3D rotation and scale can be created using methods
 * such as [`Self::from_diagonal()`], [`Self::from_quat()`], [`Self::from_axis_angle()`],
 * [`Self::from_rotation_x()`], [`Self::from_rotation_y()`], or
 * [`Self::from_rotation_z()`].
 *
 * The resulting matrices can be use to transform 3D vectors using regular vector
 * multiplication.
 *
 * Affine transformations including 2D translation, rotation and scale can be created
 * using methods such as [`Self::from_translation()`], [`Self::from_angle()`],
 * [`Self::from_scale()`] and [`Self::from_scale_angle_translation()`].
 *
 * The [`Self::transform_point2()`] and [`Self::transform_vector2()`] convenience methods
 * are provided for performing affine transforms on 2D vectors and points. These multiply
 * 2D inputs as 3D vectors with an implicit `z` value of `1` for points and `0` for
 * vectors respectively. These methods assume that `Self` contains a valid affine
 * transform.
 */
export class DMat3 {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m02
   * @param {number} m10
   * @param {number} m11
   * @param {number} m12
   * @param {number} m20
   * @param {number} m21
   * @param {number} m22
   */
  constructor(m00: number, m01: number, m02: number, m10: number, m11: number, m12: number, m20: number, m21: number, m22: number);
  x_axis: DVec3;
  y_axis: DVec3;
  z_axis: DVec3;
}
/**
 * A 4x4 column major matrix.
 *
 * This 4x4 matrix type features convenience methods for creating and using affine transforms and
 * perspective projections. If you are primarily dealing with 3D affine transformations
 * considering using [`DAffine3`](crate::DAffine3) which is faster than a 4x4 matrix
 * for some affine operations.
 *
 * Affine transformations including 3D translation, rotation and scale can be created
 * using methods such as [`Self::from_translation()`], [`Self::from_quat()`],
 * [`Self::from_scale()`] and [`Self::from_scale_rotation_translation()`].
 *
 * Orthographic projections can be created using the methods [`Self::orthographic_lh()`] for
 * left-handed coordinate systems and [`Self::orthographic_rh()`] for right-handed
 * systems. The resulting matrix is also an affine transformation.
 *
 * The [`Self::transform_point3()`] and [`Self::transform_vector3()`] convenience methods
 * are provided for performing affine transformations on 3D vectors and points. These
 * multiply 3D inputs as 4D vectors with an implicit `w` value of `1` for points and `0`
 * for vectors respectively. These methods assume that `Self` contains a valid affine
 * transform.
 *
 * Perspective projections can be created using methods such as
 * [`Self::perspective_lh()`], [`Self::perspective_infinite_lh()`] and
 * [`Self::perspective_infinite_reverse_lh()`] for left-handed co-ordinate systems and
 * [`Self::perspective_rh()`], [`Self::perspective_infinite_rh()`] and
 * [`Self::perspective_infinite_reverse_rh()`] for right-handed co-ordinate systems.
 *
 * The resulting perspective project can be use to transform 3D vectors as points with
 * perspective correction using the [`Self::project_point3()`] convenience method.
 */
export class DMat4 {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m02
   * @param {number} m03
   * @param {number} m10
   * @param {number} m11
   * @param {number} m12
   * @param {number} m13
   * @param {number} m20
   * @param {number} m21
   * @param {number} m22
   * @param {number} m23
   * @param {number} m30
   * @param {number} m31
   * @param {number} m32
   * @param {number} m33
   */
  constructor(m00: number, m01: number, m02: number, m03: number, m10: number, m11: number, m12: number, m13: number, m20: number, m21: number, m22: number, m23: number, m30: number, m31: number, m32: number, m33: number);
  w_axis: DVec4;
  x_axis: DVec4;
  y_axis: DVec4;
  z_axis: DVec4;
}
/**
 * A quaternion representing an orientation.
 *
 * This quaternion is intended to be of unit length but may denormalize due to
 * floating point "error creep" which can occur when successive quaternion
 * operations are applied.
 */
export class DQuat {
  free(): void;
  w: number;
  x: number;
  y: number;
  z: number;
}
/**
 * A 2-dimensional vector.
 */
export class DVec2 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  x: number;
  y: number;
}
/**
 * A 3-dimensional vector.
 */
export class DVec3 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 4-dimensional vector.
 */
export class DVec4 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} w
   */
  constructor(x: number, y: number, z: number, w: number);
  w: number;
  x: number;
  y: number;
  z: number;
}
export class Drag {
  free(): void;
  initial_impulse_vertical: boolean;
  start: CellCoord;
}
/**
 * An Editor represents the underlying 'state' of an edit session, including the buffer data,
 * transient buffers, masks, tools, and active tool states. It can be thought of as an active
 * 'file'. It does not include anything having to do with the presentation of the editor, like
 * cameras, viewports, and so on.
 */
export class Editor {
  free(): void;
  constructor();
  /**
   * @param {IoState} io_state
   * @param {Camera} camera
   * @returns {EditorDispatchResult}
   */
  dispatch_event(io_state: IoState, camera: Camera): EditorDispatchResult;
/**
 * The active buffer that dispatched input will be rendered to (like drawing).
 * This is used as the base for rendering (with mouse-follow stacked on top of it).
 */
  buffer: Buffer;
/**
 * The last used cursor location
 */
  cursor_coord?: CellCoord;
/**
 * The CSS style that should be applied to the cursor.
 */
  cursor_style: string;
/**
 * The current render mask applied to the buffer.
 */
  mask: Mask;
/**
 * The selected (visual mode) cells
 */
  selection: Selection;
}
export class EditorDispatchResult {
  free(): void;
  buffer_persist?: Buffer;
  tools_persist: (ToolPersist)[];
}
/**
 * A 2-dimensional vector.
 */
export class I16Vec2 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  x: number;
  y: number;
}
/**
 * A 3-dimensional vector.
 */
export class I16Vec3 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 4-dimensional vector.
 */
export class I16Vec4 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} w
   */
  constructor(x: number, y: number, z: number, w: number);
  w: number;
  x: number;
  y: number;
  z: number;
}
/**
 * A 2-dimensional vector.
 */
export class I64Vec2 {
  free(): void;
  /**
   * @param {bigint} x
   * @param {bigint} y
   */
  constructor(x: bigint, y: bigint);
  x: bigint;
  y: bigint;
}
/**
 * A 3-dimensional vector.
 */
export class I64Vec3 {
  free(): void;
  /**
   * @param {bigint} x
   * @param {bigint} y
   * @param {bigint} z
   */
  constructor(x: bigint, y: bigint, z: bigint);
  x: bigint;
  y: bigint;
  z: bigint;
}
/**
 * A 4-dimensional vector.
 */
export class I64Vec4 {
  free(): void;
  /**
   * @param {bigint} x
   * @param {bigint} y
   * @param {bigint} z
   * @param {bigint} w
   */
  constructor(x: bigint, y: bigint, z: bigint, w: bigint);
  w: bigint;
  x: bigint;
  y: bigint;
  z: bigint;
}
/**
 * A 2-dimensional vector.
 */
export class IVec2 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  x: number;
  y: number;
}
/**
 * A 3-dimensional vector.
 */
export class IVec3 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 4-dimensional vector.
 */
export class IVec4 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} w
   */
  constructor(x: number, y: number, z: number, w: number);
  w: number;
  x: number;
  y: number;
  z: number;
}
export class IoState {
  free(): void;
  constructor();
  /**
   * @param {KeyboardEvent} e
   */
  event_key_down(e: KeyboardEvent): void;
  /**
   * @param {KeyboardEvent} e
   */
  event_key_up(e: KeyboardEvent): void;
  /**
   * @param {MouseEvent} e
   * @param {Camera} camera
   */
  event_mouse(e: MouseEvent, camera: Camera): void;
  /**
   * @param {boolean} presence
   */
  event_mouse_presence(presence: boolean): void;
  /**
   * @param {WheelEvent} e
   */
  event_wheel(e: WheelEvent): void;
  /**
   * @param {string} key
   * @returns {BoolState}
   */
  get_key(key: string): BoolState;
  /**
   * @param {string} key_code
   * @returns {BoolState}
   */
  get_key_code(key_code: string): BoolState;
  /**
   * @returns {(CellCoord)[]}
   */
  get_drag_path(): (CellCoord)[];
  cell: CellCoord;
  drag?: Drag;
  hovered: BoolState;
  keys: (KeyState)[];
  primary: BoolState;
  screen_point: Vec2;
  scroll_delta_y: number;
  secondary: BoolState;
}
export class KeyState {
  free(): void;
  key: string;
  key_code: string;
  state: BoolState;
}
/**
 * Much like a Buffer, except lacking any undo or transaction support. Designed to 'overlay' a
 * buffer, activating various atoms. Any active atom that does not overlay a cell is considered
 * undefined behavior.
 */
export class Mask {
  free(): void;
}
/**
 * A 2x2 column major matrix.
 */
export class Mat2 {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m10
   * @param {number} m11
   */
  constructor(m00: number, m01: number, m10: number, m11: number);
  x_axis: Vec2;
  y_axis: Vec2;
}
/**
 * A 3x3 column major matrix.
 *
 * This 3x3 matrix type features convenience methods for creating and using linear and
 * affine transformations. If you are primarily dealing with 2D affine transformations the
 * [`Affine2`](crate::Affine2) type is much faster and more space efficient than
 * using a 3x3 matrix.
 *
 * Linear transformations including 3D rotation and scale can be created using methods
 * such as [`Self::from_diagonal()`], [`Self::from_quat()`], [`Self::from_axis_angle()`],
 * [`Self::from_rotation_x()`], [`Self::from_rotation_y()`], or
 * [`Self::from_rotation_z()`].
 *
 * The resulting matrices can be use to transform 3D vectors using regular vector
 * multiplication.
 *
 * Affine transformations including 2D translation, rotation and scale can be created
 * using methods such as [`Self::from_translation()`], [`Self::from_angle()`],
 * [`Self::from_scale()`] and [`Self::from_scale_angle_translation()`].
 *
 * The [`Self::transform_point2()`] and [`Self::transform_vector2()`] convenience methods
 * are provided for performing affine transforms on 2D vectors and points. These multiply
 * 2D inputs as 3D vectors with an implicit `z` value of `1` for points and `0` for
 * vectors respectively. These methods assume that `Self` contains a valid affine
 * transform.
 */
export class Mat3 {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m02
   * @param {number} m10
   * @param {number} m11
   * @param {number} m12
   * @param {number} m20
   * @param {number} m21
   * @param {number} m22
   */
  constructor(m00: number, m01: number, m02: number, m10: number, m11: number, m12: number, m20: number, m21: number, m22: number);
  x_axis: Vec3;
  y_axis: Vec3;
  z_axis: Vec3;
}
/**
 * A 3x3 column major matrix.
 *
 * This 3x3 matrix type features convenience methods for creating and using linear and
 * affine transformations. If you are primarily dealing with 2D affine transformations the
 * [`Affine2`](crate::Affine2) type is much faster and more space efficient than
 * using a 3x3 matrix.
 *
 * Linear transformations including 3D rotation and scale can be created using methods
 * such as [`Self::from_diagonal()`], [`Self::from_quat()`], [`Self::from_axis_angle()`],
 * [`Self::from_rotation_x()`], [`Self::from_rotation_y()`], or
 * [`Self::from_rotation_z()`].
 *
 * The resulting matrices can be use to transform 3D vectors using regular vector
 * multiplication.
 *
 * Affine transformations including 2D translation, rotation and scale can be created
 * using methods such as [`Self::from_translation()`], [`Self::from_angle()`],
 * [`Self::from_scale()`] and [`Self::from_scale_angle_translation()`].
 *
 * The [`Self::transform_point2()`] and [`Self::transform_vector2()`] convenience methods
 * are provided for performing affine transforms on 2D vectors and points. These multiply
 * 2D inputs as 3D vectors with an implicit `z` value of `1` for points and `0` for
 * vectors respectively. These methods assume that `Self` contains a valid affine
 * transform.
 */
export class Mat3A {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m02
   * @param {number} m10
   * @param {number} m11
   * @param {number} m12
   * @param {number} m20
   * @param {number} m21
   * @param {number} m22
   */
  constructor(m00: number, m01: number, m02: number, m10: number, m11: number, m12: number, m20: number, m21: number, m22: number);
  x_axis: Vec3A;
  y_axis: Vec3A;
  z_axis: Vec3A;
}
/**
 * A 4x4 column major matrix.
 *
 * This 4x4 matrix type features convenience methods for creating and using affine transforms and
 * perspective projections. If you are primarily dealing with 3D affine transformations
 * considering using [`Affine3A`](crate::Affine3A) which is faster than a 4x4 matrix
 * for some affine operations.
 *
 * Affine transformations including 3D translation, rotation and scale can be created
 * using methods such as [`Self::from_translation()`], [`Self::from_quat()`],
 * [`Self::from_scale()`] and [`Self::from_scale_rotation_translation()`].
 *
 * Orthographic projections can be created using the methods [`Self::orthographic_lh()`] for
 * left-handed coordinate systems and [`Self::orthographic_rh()`] for right-handed
 * systems. The resulting matrix is also an affine transformation.
 *
 * The [`Self::transform_point3()`] and [`Self::transform_vector3()`] convenience methods
 * are provided for performing affine transformations on 3D vectors and points. These
 * multiply 3D inputs as 4D vectors with an implicit `w` value of `1` for points and `0`
 * for vectors respectively. These methods assume that `Self` contains a valid affine
 * transform.
 *
 * Perspective projections can be created using methods such as
 * [`Self::perspective_lh()`], [`Self::perspective_infinite_lh()`] and
 * [`Self::perspective_infinite_reverse_lh()`] for left-handed co-ordinate systems and
 * [`Self::perspective_rh()`], [`Self::perspective_infinite_rh()`] and
 * [`Self::perspective_infinite_reverse_rh()`] for right-handed co-ordinate systems.
 *
 * The resulting perspective project can be use to transform 3D vectors as points with
 * perspective correction using the [`Self::project_point3()`] convenience method.
 */
export class Mat4 {
  free(): void;
  /**
   * @param {number} m00
   * @param {number} m01
   * @param {number} m02
   * @param {number} m03
   * @param {number} m10
   * @param {number} m11
   * @param {number} m12
   * @param {number} m13
   * @param {number} m20
   * @param {number} m21
   * @param {number} m22
   * @param {number} m23
   * @param {number} m30
   * @param {number} m31
   * @param {number} m32
   * @param {number} m33
   */
  constructor(m00: number, m01: number, m02: number, m03: number, m10: number, m11: number, m12: number, m13: number, m20: number, m21: number, m22: number, m23: number, m30: number, m31: number, m32: number, m33: number);
  w_axis: Vec4;
  x_axis: Vec4;
  y_axis: Vec4;
  z_axis: Vec4;
}
/**
 * NormalizedCell exists purely as a programming convenience, especially for painting. When editing
 * cells it's easier to deal with the cell as a single struct, instead of as a collection of [0, 4]
 * Atoms. NormalizedCells should be treated as transient and not stored anywhere.
 */
export class NormalizedCell {
  free(): void;
  metal: Metal;
  si: Silicon;
}
export class Pin {
  free(): void;
  /**
   * @param {CellCoord} cell_coord
   * @param {boolean} trigger
   */
  constructor(cell_coord: CellCoord, trigger: boolean);
  cell_coord: CellCoord;
  si_input_high: boolean;
  si_output_high: boolean;
  trigger: boolean;
}
/**
 * Represents the various placements of Metal and Si within a Cell, including the 4 cardinal
 * directions, and the center "self" location (which is implicit when any cardinal direction is
 * set, but can also stand alone).
 */
export class Placement {
  free(): void;
  down: boolean;
  left: boolean;
  right: boolean;
  up: boolean;
}
/**
 * A quaternion representing an orientation.
 *
 * This quaternion is intended to be of unit length but may denormalize due to
 * floating point "error creep" which can occur when successive quaternion
 * operations are applied.
 */
export class Quat {
  free(): void;
  w: number;
  x: number;
  y: number;
  z: number;
}
export class Selection {
  free(): void;
  lower_left: CellCoord;
  upper_right: CellCoord;
}
export class Socket {
  free(): void;
  /**
   * @param {(Pin)[]} pins
   * @param {boolean} always_update
   * @param {Function} update_callback
   */
  constructor(pins: (Pin)[], always_update: boolean, update_callback: Function);
  always_update: boolean;
}
export class ToolPersist {
  free(): void;
  serialized_state: Uint8Array;
  tool_name: string;
}
/**
 * A 2-dimensional vector.
 */
export class U16Vec2 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  x: number;
  y: number;
}
/**
 * A 3-dimensional vector.
 */
export class U16Vec3 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 4-dimensional vector.
 */
export class U16Vec4 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} w
   */
  constructor(x: number, y: number, z: number, w: number);
  w: number;
  x: number;
  y: number;
  z: number;
}
/**
 * A 2-dimensional vector.
 */
export class U64Vec2 {
  free(): void;
  /**
   * @param {bigint} x
   * @param {bigint} y
   */
  constructor(x: bigint, y: bigint);
  x: bigint;
  y: bigint;
}
/**
 * A 3-dimensional vector.
 */
export class U64Vec3 {
  free(): void;
  /**
   * @param {bigint} x
   * @param {bigint} y
   * @param {bigint} z
   */
  constructor(x: bigint, y: bigint, z: bigint);
  x: bigint;
  y: bigint;
  z: bigint;
}
/**
 * A 4-dimensional vector.
 */
export class U64Vec4 {
  free(): void;
  /**
   * @param {bigint} x
   * @param {bigint} y
   * @param {bigint} z
   * @param {bigint} w
   */
  constructor(x: bigint, y: bigint, z: bigint, w: bigint);
  w: bigint;
  x: bigint;
  y: bigint;
  z: bigint;
}
/**
 * Universal Packed Cell format stores each cell as a bit packed [u8; 4], ready for direct blitting
 * to a GPU RGBu8 texture. Stored as [u8; 4] instead of u32 for endian agnosticism during blitting.
 * Does not encode BufferMask data. The first 16 bits are also encoded as part of Blueprint
 * serialization.
 */
export class UPC {
  free(): void;
  /**
   * @returns {NormalizedCell}
   */
  normalize(): NormalizedCell;
  /**
   * @param {UPC} upc
   * @returns {UPC}
   */
  static denormalize(upc: UPC): UPC;
}
/**
 * A 2-dimensional vector.
 */
export class UVec2 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  x: number;
  y: number;
}
/**
 * A 3-dimensional vector.
 */
export class UVec3 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 4-dimensional vector.
 */
export class UVec4 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} w
   */
  constructor(x: number, y: number, z: number, w: number);
  w: number;
  x: number;
  y: number;
  z: number;
}
/**
 * A 2-dimensional vector.
 */
export class Vec2 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x: number, y: number);
  x: number;
  y: number;
}
/**
 * A 3-dimensional vector.
 */
export class Vec3 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 3-dimensional vector.
 *
 * SIMD vector types are used for storage on supported platforms for better
 * performance than the [`Vec3`] type.
 *
 * It is possible to convert between [`Vec3`] and [`Vec3A`] types using [`From`]
 * or [`Into`] trait implementations.
 *
 * This type is 16 byte aligned.
 */
export class Vec3A {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   */
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
 * A 4-dimensional vector.
 */
export class Vec4 {
  free(): void;
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} w
   */
  constructor(x: number, y: number, z: number, w: number);
  w: number;
  x: number;
  y: number;
  z: number;
}
/**
 * Represents only the presentation state of an on or off screen viewport for rendering.
 */
export class Viewport {
  free(): void;
  /**
   * @param {HTMLCanvasElement} canvas
   */
  constructor(canvas: HTMLCanvasElement);
  /**
   * @param {Camera} camera
   * @param {Editor} editor
   */
  draw(camera: Camera, editor: Editor): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly upc_normalize: (a: number) => number;
  readonly upc_denormalize: (a: number) => number;
  readonly __wbg_normalizedcell_free: (a: number, b: number) => void;
  readonly __wbg_get_normalizedcell_metal: (a: number) => number;
  readonly __wbg_set_normalizedcell_metal: (a: number, b: number) => void;
  readonly __wbg_get_normalizedcell_si: (a: number) => number;
  readonly __wbg_set_normalizedcell_si: (a: number, b: number) => void;
  readonly __wbg_placement_free: (a: number, b: number) => void;
  readonly __wbg_get_placement_up: (a: number) => number;
  readonly __wbg_set_placement_up: (a: number, b: number) => void;
  readonly __wbg_get_placement_right: (a: number) => number;
  readonly __wbg_set_placement_right: (a: number, b: number) => void;
  readonly __wbg_get_placement_down: (a: number) => number;
  readonly __wbg_set_placement_down: (a: number, b: number) => void;
  readonly __wbg_get_placement_left: (a: number) => number;
  readonly __wbg_set_placement_left: (a: number, b: number) => void;
  readonly __wbg_upc_free: (a: number, b: number) => void;
  readonly __wbg_viewport_free: (a: number, b: number) => void;
  readonly viewport_new: (a: number) => number;
  readonly viewport_draw: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_camera_free: (a: number, b: number) => void;
  readonly __wbg_get_camera_translation: (a: number) => number;
  readonly __wbg_set_camera_translation: (a: number, b: number) => void;
  readonly __wbg_get_camera_scale: (a: number) => number;
  readonly __wbg_set_camera_scale: (a: number, b: number) => void;
  readonly __wbg_get_camera_size: (a: number) => number;
  readonly __wbg_set_camera_size: (a: number, b: number) => void;
  readonly camera_new_translation_scale: (a: number, b: number) => number;
  readonly camera_project_screen_point_to_world: (a: number, b: number) => number;
  readonly camera_project_screen_point_to_cell: (a: number, b: number) => number;
  readonly camera_project_cell_coord_to_screen_point: (a: number, b: number) => number;
  readonly __wbg_buffer_free: (a: number, b: number) => void;
  readonly buffer_new: () => number;
  readonly buffer_get_cell: (a: number, b: number) => number;
  readonly buffer_set_cell: (a: number, b: number, c: number) => void;
  readonly buffer_clone_selection: (a: number, b: number, c: number) => number;
  readonly buffer_paste_at: (a: number, b: number, c: number) => void;
  readonly buffer_rotate_to_new: (a: number) => number;
  readonly buffer_mirror_to_new: (a: number) => number;
  readonly buffer_fix_all_cells: (a: number) => void;
  readonly buffer_cell_count: (a: number) => number;
  readonly __wbg_pin_free: (a: number, b: number) => void;
  readonly __wbg_get_pin_cell_coord: (a: number) => number;
  readonly __wbg_set_pin_cell_coord: (a: number, b: number) => void;
  readonly __wbg_get_pin_trigger: (a: number) => number;
  readonly __wbg_set_pin_trigger: (a: number, b: number) => void;
  readonly __wbg_get_pin_si_output_high: (a: number) => number;
  readonly __wbg_set_pin_si_output_high: (a: number, b: number) => void;
  readonly __wbg_get_pin_si_input_high: (a: number) => number;
  readonly __wbg_set_pin_si_input_high: (a: number, b: number) => void;
  readonly __wbg_socket_free: (a: number, b: number) => void;
  readonly __wbg_get_socket_always_update: (a: number) => number;
  readonly __wbg_set_socket_always_update: (a: number, b: number) => void;
  readonly pin_new: (a: number, b: number) => number;
  readonly socket_new: (a: number, b: number, c: number, d: number) => number;
  readonly buffer_to_base64_string: (a: number, b: number) => void;
  readonly buffer_to_bytes: (a: number, b: number) => void;
  readonly buffer_from_base64_string: (a: number, b: number, c: number) => void;
  readonly buffer_from_bytes: (a: number, b: number, c: number) => void;
  readonly __wbg_boolstate_free: (a: number, b: number) => void;
  readonly __wbg_get_boolstate_clicked: (a: number) => number;
  readonly __wbg_set_boolstate_clicked: (a: number, b: number) => void;
  readonly __wbg_get_boolstate_down: (a: number) => number;
  readonly __wbg_set_boolstate_down: (a: number, b: number) => void;
  readonly __wbg_get_boolstate_released: (a: number) => number;
  readonly __wbg_set_boolstate_released: (a: number, b: number) => void;
  readonly __wbg_keystate_free: (a: number, b: number) => void;
  readonly __wbg_get_keystate_key_code: (a: number, b: number) => void;
  readonly __wbg_set_keystate_key_code: (a: number, b: number, c: number) => void;
  readonly __wbg_get_keystate_key: (a: number, b: number) => void;
  readonly __wbg_set_keystate_key: (a: number, b: number, c: number) => void;
  readonly __wbg_get_keystate_state: (a: number) => number;
  readonly __wbg_set_keystate_state: (a: number, b: number) => void;
  readonly __wbg_drag_free: (a: number, b: number) => void;
  readonly __wbg_get_drag_start: (a: number) => number;
  readonly __wbg_set_drag_start: (a: number, b: number) => void;
  readonly __wbg_get_drag_initial_impulse_vertical: (a: number) => number;
  readonly __wbg_set_drag_initial_impulse_vertical: (a: number, b: number) => void;
  readonly __wbg_iostate_free: (a: number, b: number) => void;
  readonly __wbg_get_iostate_hovered: (a: number) => number;
  readonly __wbg_set_iostate_hovered: (a: number, b: number) => void;
  readonly __wbg_get_iostate_primary: (a: number) => number;
  readonly __wbg_set_iostate_primary: (a: number, b: number) => void;
  readonly __wbg_get_iostate_secondary: (a: number) => number;
  readonly __wbg_set_iostate_secondary: (a: number, b: number) => void;
  readonly __wbg_get_iostate_drag: (a: number) => number;
  readonly __wbg_set_iostate_drag: (a: number, b: number) => void;
  readonly __wbg_get_iostate_keys: (a: number, b: number) => void;
  readonly __wbg_set_iostate_keys: (a: number, b: number, c: number) => void;
  readonly __wbg_get_iostate_screen_point: (a: number) => number;
  readonly __wbg_set_iostate_screen_point: (a: number, b: number) => void;
  readonly __wbg_get_iostate_cell: (a: number) => number;
  readonly __wbg_set_iostate_cell: (a: number, b: number) => void;
  readonly __wbg_get_iostate_scroll_delta_y: (a: number) => number;
  readonly __wbg_set_iostate_scroll_delta_y: (a: number, b: number) => void;
  readonly iostate_new: () => number;
  readonly iostate_event_key_down: (a: number, b: number) => void;
  readonly iostate_event_key_up: (a: number, b: number) => void;
  readonly iostate_event_mouse: (a: number, b: number, c: number) => void;
  readonly iostate_event_mouse_presence: (a: number, b: number) => void;
  readonly iostate_event_wheel: (a: number, b: number) => void;
  readonly iostate_get_key: (a: number, b: number, c: number) => number;
  readonly iostate_get_key_code: (a: number, b: number, c: number) => number;
  readonly iostate_get_drag_path: (a: number, b: number) => void;
  readonly buffer_draw_si: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly buffer_draw_metal: (a: number, b: number, c: number, d: number) => void;
  readonly buffer_clear_si: (a: number, b: number, c: number, d: number) => void;
  readonly buffer_clear_metal: (a: number, b: number, c: number, d: number) => void;
  readonly buffer_draw_via: (a: number, b: number) => void;
  readonly buffer_clear_selection: (a: number, b: number) => void;
  readonly buffer_clear_selection_border: (a: number, b: number) => void;
  readonly buffer_draw_si_link: (a: number, b: number, c: number, d: number) => void;
  readonly buffer_draw_metal_link: (a: number, b: number, c: number) => void;
  readonly buffer_clear_cell_si: (a: number, b: number) => void;
  readonly buffer_clear_cell_metal: (a: number, b: number) => void;
  readonly __wbg_cellcoord_free: (a: number, b: number) => void;
  readonly __wbg_get_cellcoord_0: (a: number) => number;
  readonly __wbg_set_cellcoord_0: (a: number, b: number) => void;
  readonly cellcoord__wasm_ctor: (a: number, b: number) => number;
  readonly __wbg_editor_free: (a: number, b: number) => void;
  readonly __wbg_get_editor_buffer: (a: number) => number;
  readonly __wbg_set_editor_buffer: (a: number, b: number) => void;
  readonly __wbg_get_editor_mask: (a: number) => number;
  readonly __wbg_set_editor_mask: (a: number, b: number) => void;
  readonly __wbg_get_editor_selection: (a: number) => number;
  readonly __wbg_set_editor_selection: (a: number, b: number) => void;
  readonly __wbg_get_editor_cursor_coord: (a: number) => number;
  readonly __wbg_set_editor_cursor_coord: (a: number, b: number) => void;
  readonly __wbg_get_editor_cursor_style: (a: number, b: number) => void;
  readonly __wbg_set_editor_cursor_style: (a: number, b: number, c: number) => void;
  readonly __wbg_editordispatchresult_free: (a: number, b: number) => void;
  readonly __wbg_get_editordispatchresult_buffer_persist: (a: number) => number;
  readonly __wbg_set_editordispatchresult_buffer_persist: (a: number, b: number) => void;
  readonly __wbg_get_editordispatchresult_tools_persist: (a: number, b: number) => void;
  readonly __wbg_set_editordispatchresult_tools_persist: (a: number, b: number, c: number) => void;
  readonly __wbg_toolpersist_free: (a: number, b: number) => void;
  readonly __wbg_get_toolpersist_tool_name: (a: number, b: number) => void;
  readonly __wbg_set_toolpersist_tool_name: (a: number, b: number, c: number) => void;
  readonly __wbg_get_toolpersist_serialized_state: (a: number, b: number) => void;
  readonly __wbg_set_toolpersist_serialized_state: (a: number, b: number, c: number) => void;
  readonly editor_new: () => number;
  readonly editor_dispatch_event: (a: number, b: number, c: number) => number;
  readonly import_legacy_blueprint: (a: number, b: number, c: number) => void;
  readonly __wbg_compilerresults_free: (a: number, b: number) => void;
  readonly __wbg_atom_free: (a: number, b: number) => void;
  readonly __wbg_get_atom_coord: (a: number) => number;
  readonly __wbg_set_atom_coord: (a: number, b: number) => void;
  readonly __wbg_get_atom_part: (a: number) => number;
  readonly __wbg_set_atom_part: (a: number, b: number) => void;
  readonly compilerresults_from_buffer: (a: number) => number;
  readonly compilerresults_get_trace_atoms: (a: number, b: number, c: number) => void;
  readonly __wbg_mask_free: (a: number, b: number) => void;
  readonly __wbg_selection_free: (a: number, b: number) => void;
  readonly __wbg_get_selection_lower_left: (a: number) => number;
  readonly __wbg_set_selection_lower_left: (a: number, b: number) => void;
  readonly __wbg_get_selection_upper_right: (a: number) => number;
  readonly __wbg_set_selection_upper_right: (a: number, b: number) => void;
  readonly main: () => void;
  readonly __wbg_get_vec2_x: (a: number) => number;
  readonly __wbg_set_vec2_x: (a: number, b: number) => void;
  readonly __wbg_get_vec2_y: (a: number) => number;
  readonly __wbg_set_vec2_y: (a: number, b: number) => void;
  readonly vec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_vec3a_free: (a: number, b: number) => void;
  readonly __wbg_get_vec3a_x: (a: number) => number;
  readonly __wbg_set_vec3a_x: (a: number, b: number) => void;
  readonly __wbg_get_vec3a_y: (a: number) => number;
  readonly __wbg_set_vec3a_y: (a: number, b: number) => void;
  readonly __wbg_get_vec3a_z: (a: number) => number;
  readonly __wbg_set_vec3a_z: (a: number, b: number) => void;
  readonly vec3a_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_get_vec4_w: (a: number) => number;
  readonly __wbg_set_vec4_w: (a: number, b: number) => void;
  readonly vec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_ivec2_free: (a: number, b: number) => void;
  readonly __wbg_get_ivec2_x: (a: number) => number;
  readonly __wbg_set_ivec2_x: (a: number, b: number) => void;
  readonly __wbg_get_ivec2_y: (a: number) => number;
  readonly __wbg_set_ivec2_y: (a: number, b: number) => void;
  readonly ivec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_ivec4_free: (a: number, b: number) => void;
  readonly __wbg_get_ivec4_z: (a: number) => number;
  readonly __wbg_set_ivec4_z: (a: number, b: number) => void;
  readonly __wbg_get_ivec4_w: (a: number) => number;
  readonly __wbg_set_ivec4_w: (a: number, b: number) => void;
  readonly ivec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_get_vec4_x: (a: number) => number;
  readonly __wbg_get_vec4_y: (a: number) => number;
  readonly __wbg_get_vec4_z: (a: number) => number;
  readonly __wbg_get_ivec4_x: (a: number) => number;
  readonly __wbg_get_ivec4_y: (a: number) => number;
  readonly __wbg_vec4_free: (a: number, b: number) => void;
  readonly __wbg_vec2_free: (a: number, b: number) => void;
  readonly __wbg_set_vec4_x: (a: number, b: number) => void;
  readonly __wbg_set_vec4_y: (a: number, b: number) => void;
  readonly __wbg_set_vec4_z: (a: number, b: number) => void;
  readonly __wbg_set_ivec4_x: (a: number, b: number) => void;
  readonly __wbg_set_ivec4_y: (a: number, b: number) => void;
  readonly __wbg_u64vec2_free: (a: number, b: number) => void;
  readonly __wbg_get_u64vec2_x: (a: number) => number;
  readonly __wbg_set_u64vec2_x: (a: number, b: number) => void;
  readonly __wbg_get_u64vec2_y: (a: number) => number;
  readonly __wbg_set_u64vec2_y: (a: number, b: number) => void;
  readonly u64vec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_u64vec4_free: (a: number, b: number) => void;
  readonly __wbg_get_u64vec4_z: (a: number) => number;
  readonly __wbg_set_u64vec4_z: (a: number, b: number) => void;
  readonly __wbg_get_u64vec4_w: (a: number) => number;
  readonly __wbg_set_u64vec4_w: (a: number, b: number) => void;
  readonly u64vec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_get_u64vec4_x: (a: number) => number;
  readonly __wbg_get_u64vec4_y: (a: number) => number;
  readonly __wbg_set_u64vec4_x: (a: number, b: number) => void;
  readonly __wbg_set_u64vec4_y: (a: number, b: number) => void;
  readonly __wbg_dvec2_free: (a: number, b: number) => void;
  readonly __wbg_get_dvec2_x: (a: number) => number;
  readonly __wbg_set_dvec2_x: (a: number, b: number) => void;
  readonly __wbg_get_dvec2_y: (a: number) => number;
  readonly __wbg_set_dvec2_y: (a: number, b: number) => void;
  readonly dvec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_dvec4_free: (a: number, b: number) => void;
  readonly __wbg_get_dvec4_z: (a: number) => number;
  readonly __wbg_set_dvec4_z: (a: number, b: number) => void;
  readonly __wbg_get_dvec4_w: (a: number) => number;
  readonly __wbg_set_dvec4_w: (a: number, b: number) => void;
  readonly dvec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_get_i64vec2_x: (a: number) => number;
  readonly __wbg_set_i64vec2_x: (a: number, b: number) => void;
  readonly __wbg_get_i64vec2_y: (a: number) => number;
  readonly __wbg_set_i64vec2_y: (a: number, b: number) => void;
  readonly i64vec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_u64vec3_free: (a: number, b: number) => void;
  readonly __wbg_get_u64vec3_z: (a: number) => number;
  readonly __wbg_set_u64vec3_z: (a: number, b: number) => void;
  readonly u64vec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_get_dvec4_x: (a: number) => number;
  readonly __wbg_get_dvec4_y: (a: number) => number;
  readonly __wbg_get_u64vec3_x: (a: number) => number;
  readonly __wbg_get_u64vec3_y: (a: number) => number;
  readonly __wbg_i64vec2_free: (a: number, b: number) => void;
  readonly __wbg_set_dvec4_x: (a: number, b: number) => void;
  readonly __wbg_set_dvec4_y: (a: number, b: number) => void;
  readonly __wbg_set_u64vec3_x: (a: number, b: number) => void;
  readonly __wbg_set_u64vec3_y: (a: number, b: number) => void;
  readonly __wbg_mat4_free: (a: number, b: number) => void;
  readonly __wbg_get_mat4_x_axis: (a: number) => number;
  readonly __wbg_set_mat4_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat4_y_axis: (a: number) => number;
  readonly __wbg_set_mat4_y_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat4_z_axis: (a: number) => number;
  readonly __wbg_set_mat4_z_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat4_w_axis: (a: number) => number;
  readonly __wbg_set_mat4_w_axis: (a: number, b: number) => void;
  readonly mat4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number) => number;
  readonly __wbg_i16vec2_free: (a: number, b: number) => void;
  readonly __wbg_get_i16vec2_x: (a: number) => number;
  readonly __wbg_set_i16vec2_x: (a: number, b: number) => void;
  readonly __wbg_get_i16vec2_y: (a: number) => number;
  readonly __wbg_set_i16vec2_y: (a: number, b: number) => void;
  readonly i16vec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_i16vec3_free: (a: number, b: number) => void;
  readonly __wbg_get_i16vec3_z: (a: number) => number;
  readonly __wbg_set_i16vec3_z: (a: number, b: number) => void;
  readonly i16vec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_get_i16vec4_w: (a: number) => number;
  readonly __wbg_set_i16vec4_w: (a: number, b: number) => void;
  readonly i16vec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_uvec2_free: (a: number, b: number) => void;
  readonly __wbg_get_uvec2_x: (a: number) => number;
  readonly __wbg_set_uvec2_x: (a: number, b: number) => void;
  readonly __wbg_get_uvec2_y: (a: number) => number;
  readonly __wbg_set_uvec2_y: (a: number, b: number) => void;
  readonly uvec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_set_i16vec3_x: (a: number, b: number) => void;
  readonly __wbg_set_i16vec3_y: (a: number, b: number) => void;
  readonly __wbg_set_i16vec4_x: (a: number, b: number) => void;
  readonly __wbg_set_i16vec4_y: (a: number, b: number) => void;
  readonly __wbg_set_i16vec4_z: (a: number, b: number) => void;
  readonly __wbg_get_i16vec3_x: (a: number) => number;
  readonly __wbg_get_i16vec3_y: (a: number) => number;
  readonly __wbg_get_i16vec4_x: (a: number) => number;
  readonly __wbg_get_i16vec4_y: (a: number) => number;
  readonly __wbg_get_i16vec4_z: (a: number) => number;
  readonly __wbg_i16vec4_free: (a: number, b: number) => void;
  readonly __wbg_mat3_free: (a: number, b: number) => void;
  readonly __wbg_get_mat3_x_axis: (a: number) => number;
  readonly __wbg_set_mat3_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat3_y_axis: (a: number) => number;
  readonly __wbg_set_mat3_y_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat3_z_axis: (a: number) => number;
  readonly __wbg_set_mat3_z_axis: (a: number, b: number) => void;
  readonly mat3_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly __wbg_ivec3_free: (a: number, b: number) => void;
  readonly __wbg_get_ivec3_x: (a: number) => number;
  readonly __wbg_set_ivec3_x: (a: number, b: number) => void;
  readonly __wbg_get_ivec3_y: (a: number) => number;
  readonly __wbg_set_ivec3_y: (a: number, b: number) => void;
  readonly __wbg_get_ivec3_z: (a: number) => number;
  readonly __wbg_set_ivec3_z: (a: number, b: number) => void;
  readonly ivec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_dmat2_free: (a: number, b: number) => void;
  readonly __wbg_get_dmat2_x_axis: (a: number) => number;
  readonly __wbg_set_dmat2_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_dmat2_y_axis: (a: number) => number;
  readonly __wbg_set_dmat2_y_axis: (a: number, b: number) => void;
  readonly dmat2_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_dmat3_free: (a: number, b: number) => void;
  readonly __wbg_get_dmat3_x_axis: (a: number) => number;
  readonly __wbg_set_dmat3_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_dmat3_y_axis: (a: number) => number;
  readonly __wbg_set_dmat3_y_axis: (a: number, b: number) => void;
  readonly __wbg_get_dmat3_z_axis: (a: number) => number;
  readonly __wbg_set_dmat3_z_axis: (a: number, b: number) => void;
  readonly dmat3_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly __wbg_dquat_free: (a: number, b: number) => void;
  readonly __wbg_get_dquat_x: (a: number) => number;
  readonly __wbg_set_dquat_x: (a: number, b: number) => void;
  readonly __wbg_get_dquat_y: (a: number) => number;
  readonly __wbg_set_dquat_y: (a: number, b: number) => void;
  readonly __wbg_get_dquat_z: (a: number) => number;
  readonly __wbg_set_dquat_z: (a: number, b: number) => void;
  readonly __wbg_get_dquat_w: (a: number) => number;
  readonly __wbg_set_dquat_w: (a: number, b: number) => void;
  readonly __wbg_dvec3_free: (a: number, b: number) => void;
  readonly dvec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_uvec3_free: (a: number, b: number) => void;
  readonly __wbg_get_uvec3_x: (a: number) => number;
  readonly __wbg_set_uvec3_x: (a: number, b: number) => void;
  readonly __wbg_get_uvec3_y: (a: number) => number;
  readonly __wbg_set_uvec3_y: (a: number, b: number) => void;
  readonly __wbg_get_uvec3_z: (a: number) => number;
  readonly __wbg_set_uvec3_z: (a: number, b: number) => void;
  readonly uvec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_uvec4_free: (a: number, b: number) => void;
  readonly __wbg_get_uvec4_w: (a: number) => number;
  readonly __wbg_set_uvec4_w: (a: number, b: number) => void;
  readonly uvec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_get_dvec3_x: (a: number) => number;
  readonly __wbg_get_dvec3_y: (a: number) => number;
  readonly __wbg_get_dvec3_z: (a: number) => number;
  readonly __wbg_get_uvec4_x: (a: number) => number;
  readonly __wbg_get_uvec4_y: (a: number) => number;
  readonly __wbg_get_uvec4_z: (a: number) => number;
  readonly __wbg_set_dvec3_x: (a: number, b: number) => void;
  readonly __wbg_set_dvec3_y: (a: number, b: number) => void;
  readonly __wbg_set_dvec3_z: (a: number, b: number) => void;
  readonly __wbg_set_uvec4_x: (a: number, b: number) => void;
  readonly __wbg_set_uvec4_y: (a: number, b: number) => void;
  readonly __wbg_set_uvec4_z: (a: number, b: number) => void;
  readonly __wbg_mat2_free: (a: number, b: number) => void;
  readonly __wbg_get_mat2_x_axis: (a: number) => number;
  readonly __wbg_set_mat2_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat2_y_axis: (a: number) => number;
  readonly __wbg_set_mat2_y_axis: (a: number, b: number) => void;
  readonly mat2_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_mat3a_free: (a: number, b: number) => void;
  readonly __wbg_get_mat3a_x_axis: (a: number) => number;
  readonly __wbg_set_mat3a_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat3a_y_axis: (a: number) => number;
  readonly __wbg_set_mat3a_y_axis: (a: number, b: number) => void;
  readonly __wbg_get_mat3a_z_axis: (a: number) => number;
  readonly __wbg_set_mat3a_z_axis: (a: number, b: number) => void;
  readonly mat3a_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly __wbg_dmat4_free: (a: number, b: number) => void;
  readonly __wbg_get_dmat4_x_axis: (a: number) => number;
  readonly __wbg_set_dmat4_x_axis: (a: number, b: number) => void;
  readonly __wbg_get_dmat4_y_axis: (a: number) => number;
  readonly __wbg_set_dmat4_y_axis: (a: number, b: number) => void;
  readonly __wbg_get_dmat4_z_axis: (a: number) => number;
  readonly __wbg_set_dmat4_z_axis: (a: number, b: number) => void;
  readonly __wbg_get_dmat4_w_axis: (a: number) => number;
  readonly __wbg_set_dmat4_w_axis: (a: number, b: number) => void;
  readonly dmat4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number) => number;
  readonly __wbg_u16vec2_free: (a: number, b: number) => void;
  readonly __wbg_get_u16vec2_x: (a: number) => number;
  readonly __wbg_set_u16vec2_x: (a: number, b: number) => void;
  readonly __wbg_get_u16vec2_y: (a: number) => number;
  readonly __wbg_set_u16vec2_y: (a: number, b: number) => void;
  readonly u16vec2_wasm_bindgen_ctor: (a: number, b: number) => number;
  readonly __wbg_u16vec3_free: (a: number, b: number) => void;
  readonly __wbg_get_u16vec3_z: (a: number) => number;
  readonly __wbg_set_u16vec3_z: (a: number, b: number) => void;
  readonly u16vec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_get_u16vec4_w: (a: number) => number;
  readonly __wbg_set_u16vec4_w: (a: number, b: number) => void;
  readonly u16vec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_set_u16vec3_x: (a: number, b: number) => void;
  readonly __wbg_set_u16vec3_y: (a: number, b: number) => void;
  readonly __wbg_set_u16vec4_x: (a: number, b: number) => void;
  readonly __wbg_set_u16vec4_y: (a: number, b: number) => void;
  readonly __wbg_set_u16vec4_z: (a: number, b: number) => void;
  readonly __wbg_get_u16vec3_x: (a: number) => number;
  readonly __wbg_get_u16vec3_y: (a: number) => number;
  readonly __wbg_get_u16vec4_x: (a: number) => number;
  readonly __wbg_get_u16vec4_y: (a: number) => number;
  readonly __wbg_get_u16vec4_z: (a: number) => number;
  readonly __wbg_u16vec4_free: (a: number, b: number) => void;
  readonly __wbg_vec3_free: (a: number, b: number) => void;
  readonly __wbg_get_vec3_x: (a: number) => number;
  readonly __wbg_set_vec3_x: (a: number, b: number) => void;
  readonly __wbg_get_vec3_y: (a: number) => number;
  readonly __wbg_set_vec3_y: (a: number, b: number) => void;
  readonly __wbg_get_vec3_z: (a: number) => number;
  readonly __wbg_set_vec3_z: (a: number, b: number) => void;
  readonly vec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_quat_free: (a: number, b: number) => void;
  readonly __wbg_get_quat_x: (a: number) => number;
  readonly __wbg_set_quat_x: (a: number, b: number) => void;
  readonly __wbg_get_quat_y: (a: number) => number;
  readonly __wbg_set_quat_y: (a: number, b: number) => void;
  readonly __wbg_get_quat_z: (a: number) => number;
  readonly __wbg_set_quat_z: (a: number, b: number) => void;
  readonly __wbg_get_quat_w: (a: number) => number;
  readonly __wbg_set_quat_w: (a: number, b: number) => void;
  readonly __wbg_i64vec3_free: (a: number, b: number) => void;
  readonly __wbg_get_i64vec3_x: (a: number) => number;
  readonly __wbg_set_i64vec3_x: (a: number, b: number) => void;
  readonly __wbg_get_i64vec3_y: (a: number) => number;
  readonly __wbg_set_i64vec3_y: (a: number, b: number) => void;
  readonly __wbg_get_i64vec3_z: (a: number) => number;
  readonly __wbg_set_i64vec3_z: (a: number, b: number) => void;
  readonly i64vec3_wasm_bindgen_ctor: (a: number, b: number, c: number) => number;
  readonly __wbg_i64vec4_free: (a: number, b: number) => void;
  readonly __wbg_get_i64vec4_w: (a: number) => number;
  readonly __wbg_set_i64vec4_w: (a: number, b: number) => void;
  readonly i64vec4_wasm_bindgen_ctor: (a: number, b: number, c: number, d: number) => number;
  readonly __wbg_get_i64vec4_x: (a: number) => number;
  readonly __wbg_get_i64vec4_y: (a: number) => number;
  readonly __wbg_get_i64vec4_z: (a: number) => number;
  readonly __wbg_set_i64vec4_x: (a: number, b: number) => void;
  readonly __wbg_set_i64vec4_y: (a: number, b: number) => void;
  readonly __wbg_set_i64vec4_z: (a: number, b: number) => void;
  readonly memory: WebAssembly.Memory;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_4: (a: number) => void;
  readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
  readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
* @param {WebAssembly.Memory} memory - Deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
* @param {WebAssembly.Memory} memory - Deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
