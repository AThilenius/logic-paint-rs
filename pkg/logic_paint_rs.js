let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.buffer !== wasm.memory.buffer) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().slice(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
};

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addHeapObject(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(takeObject(mem.getUint32(i, true)));
    }
    return result;
}
/**
 * Convert a legacy blueprint JSON file into a Buffer (which can then be saved into the latest
 * format). Does not support modules, only the substrate is loaded.
 * @param {string} json_str
 * @returns {Buffer}
 */
export function import_legacy_blueprint(json_str) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.import_legacy_blueprint(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return Buffer.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

export function main() {
    wasm.main();
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export_4(addHeapObject(e));
    }
}

let cachedFloat32ArrayMemory0 = null;

function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.buffer !== wasm.memory.buffer) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

export const CellPart = Object.freeze({ Metal:0,"0":"Metal",Si:1,"1":"Si",EcUpLeft:2,"2":"EcUpLeft",EcDownRight:3,"3":"EcDownRight", });

const __wbindgen_enum_WorkerType = ["classic", "module"];

const AtomFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_atom_free(ptr >>> 0, 1));

export class Atom {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Atom.prototype);
        obj.__wbg_ptr = ptr;
        AtomFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AtomFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_atom_free(ptr, 0);
    }
    /**
     * @returns {CellCoord}
     */
    get coord() {
        const ret = wasm.__wbg_get_atom_coord(this.__wbg_ptr);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} arg0
     */
    set coord(arg0) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_atom_coord(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {CellPart}
     */
    get part() {
        const ret = wasm.__wbg_get_atom_part(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {CellPart} arg0
     */
    set part(arg0) {
        wasm.__wbg_set_atom_part(this.__wbg_ptr, arg0);
    }
}

const BoolStateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_boolstate_free(ptr >>> 0, 1));

export class BoolState {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BoolState.prototype);
        obj.__wbg_ptr = ptr;
        BoolStateFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BoolStateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_boolstate_free(ptr, 0);
    }
    /**
     * The key was just clicked this dispatch.
     * @returns {boolean}
     */
    get clicked() {
        const ret = wasm.__wbg_get_boolstate_clicked(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * The key was just clicked this dispatch.
     * @param {boolean} arg0
     */
    set clicked(arg0) {
        wasm.__wbg_set_boolstate_clicked(this.__wbg_ptr, arg0);
    }
    /**
     * The key is being held down. Can be true when `clicked` is true.
     * @returns {boolean}
     */
    get down() {
        const ret = wasm.__wbg_get_boolstate_down(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * The key is being held down. Can be true when `clicked` is true.
     * @param {boolean} arg0
     */
    set down(arg0) {
        wasm.__wbg_set_boolstate_down(this.__wbg_ptr, arg0);
    }
    /**
     * The key was just released this dispatch.
     * @returns {boolean}
     */
    get released() {
        const ret = wasm.__wbg_get_boolstate_released(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * The key was just released this dispatch.
     * @param {boolean} arg0
     */
    set released(arg0) {
        wasm.__wbg_set_boolstate_released(this.__wbg_ptr, arg0);
    }
}

const BufferFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_buffer_free(ptr >>> 0, 1));
/**
 * Buffers are an infinite grid of cells, where each cell is 4 bytes. Things are split into
 * chunks, where each chunk stores a simple Vec<u8>, and Chunks are indexed by their chunk
 * coordinate on the infinite grid. Chunks with zero non-default cells take up no memory.
 *
 * This struct is cheap to clone, as chunks are Copy-On-Write thanks to `im` HashMap. Sockets
 * however are cloned in their entirety, because they are relatively small.
 */
export class Buffer {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Buffer.prototype);
        obj.__wbg_ptr = ptr;
        BufferFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BufferFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_buffer_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.buffer_new();
        this.__wbg_ptr = ret >>> 0;
        BufferFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {CellCoord} cell_coord
     * @returns {UPC}
     */
    get_cell(cell_coord) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        const ret = wasm.buffer_get_cell(this.__wbg_ptr, ptr0);
        return UPC.__wrap(ret);
    }
    /**
     * @param {CellCoord} cell_coord
     * @param {UPC} cell
     */
    set_cell(cell_coord, cell) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        _assertClass(cell, UPC);
        var ptr1 = cell.__destroy_into_raw();
        wasm.buffer_set_cell(this.__wbg_ptr, ptr0, ptr1);
    }
    /**
     * @param {Selection} selection
     * @param {CellCoord} anchor
     * @returns {Buffer}
     */
    clone_selection(selection, anchor) {
        _assertClass(selection, Selection);
        _assertClass(anchor, CellCoord);
        var ptr0 = anchor.__destroy_into_raw();
        const ret = wasm.buffer_clone_selection(this.__wbg_ptr, selection.__wbg_ptr, ptr0);
        return Buffer.__wrap(ret);
    }
    /**
     * @param {CellCoord} cell_coord
     * @param {Buffer} buffer
     */
    paste_at(cell_coord, buffer) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        _assertClass(buffer, Buffer);
        wasm.buffer_paste_at(this.__wbg_ptr, ptr0, buffer.__wbg_ptr);
    }
    /**
     * @returns {Buffer}
     */
    rotate_to_new() {
        const ret = wasm.buffer_rotate_to_new(this.__wbg_ptr);
        return Buffer.__wrap(ret);
    }
    /**
     * @returns {Buffer}
     */
    mirror_to_new() {
        const ret = wasm.buffer_mirror_to_new(this.__wbg_ptr);
        return Buffer.__wrap(ret);
    }
    fix_all_cells() {
        wasm.buffer_fix_all_cells(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    cell_count() {
        const ret = wasm.buffer_cell_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    to_base64_string() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.buffer_to_base64_string(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export_3(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {Uint8Array}
     */
    to_bytes() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.buffer_to_bytes(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_3(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} base_64_string
     * @returns {Buffer}
     */
    static from_base64_string(base_64_string) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(base_64_string, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len0 = WASM_VECTOR_LEN;
            wasm.buffer_from_base64_string(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return Buffer.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {Uint8Array} bytes
     * @returns {Buffer}
     */
    static from_bytes(bytes) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export_1);
            const len0 = WASM_VECTOR_LEN;
            wasm.buffer_from_bytes(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return Buffer.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {CellCoord} arg0
     * @param {CellCoord} arg1
     * @param {boolean} initial_impulse_vertical
     * @param {boolean} paint_n
     */
    draw_si(arg0, arg1, initial_impulse_vertical, paint_n) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        _assertClass(arg1, CellCoord);
        var ptr1 = arg1.__destroy_into_raw();
        wasm.buffer_draw_si(this.__wbg_ptr, ptr0, ptr1, initial_impulse_vertical, paint_n);
    }
    /**
     * @param {CellCoord} arg0
     * @param {CellCoord} arg1
     * @param {boolean} initial_impulse_vertical
     */
    draw_metal(arg0, arg1, initial_impulse_vertical) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        _assertClass(arg1, CellCoord);
        var ptr1 = arg1.__destroy_into_raw();
        wasm.buffer_draw_metal(this.__wbg_ptr, ptr0, ptr1, initial_impulse_vertical);
    }
    /**
     * @param {CellCoord} arg0
     * @param {CellCoord} arg1
     * @param {boolean} initial_impulse_vertical
     */
    clear_si(arg0, arg1, initial_impulse_vertical) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        _assertClass(arg1, CellCoord);
        var ptr1 = arg1.__destroy_into_raw();
        wasm.buffer_clear_si(this.__wbg_ptr, ptr0, ptr1, initial_impulse_vertical);
    }
    /**
     * @param {CellCoord} arg0
     * @param {CellCoord} arg1
     * @param {boolean} initial_impulse_vertical
     */
    clear_metal(arg0, arg1, initial_impulse_vertical) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        _assertClass(arg1, CellCoord);
        var ptr1 = arg1.__destroy_into_raw();
        wasm.buffer_clear_metal(this.__wbg_ptr, ptr0, ptr1, initial_impulse_vertical);
    }
    /**
     * @param {CellCoord} cell_coord
     */
    draw_via(cell_coord) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        wasm.buffer_draw_via(this.__wbg_ptr, ptr0);
    }
    /**
     * @param {Selection} selection
     */
    clear_selection(selection) {
        _assertClass(selection, Selection);
        wasm.buffer_clear_selection(this.__wbg_ptr, selection.__wbg_ptr);
    }
    /**
     * @param {Selection} selection
     */
    clear_selection_border(selection) {
        _assertClass(selection, Selection);
        wasm.buffer_clear_selection_border(this.__wbg_ptr, selection.__wbg_ptr);
    }
    /**
     * @param {CellCoord | undefined} from
     * @param {CellCoord} to
     * @param {boolean} paint_n
     */
    draw_si_link(from, to, paint_n) {
        let ptr0 = 0;
        if (!isLikeNone(from)) {
            _assertClass(from, CellCoord);
            ptr0 = from.__destroy_into_raw();
        }
        _assertClass(to, CellCoord);
        var ptr1 = to.__destroy_into_raw();
        wasm.buffer_draw_si_link(this.__wbg_ptr, ptr0, ptr1, paint_n);
    }
    /**
     * @param {CellCoord | undefined} from
     * @param {CellCoord} to
     */
    draw_metal_link(from, to) {
        let ptr0 = 0;
        if (!isLikeNone(from)) {
            _assertClass(from, CellCoord);
            ptr0 = from.__destroy_into_raw();
        }
        _assertClass(to, CellCoord);
        var ptr1 = to.__destroy_into_raw();
        wasm.buffer_draw_metal_link(this.__wbg_ptr, ptr0, ptr1);
    }
    /**
     * @param {CellCoord} cell_coord
     */
    clear_cell_si(cell_coord) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        wasm.buffer_clear_cell_si(this.__wbg_ptr, ptr0);
    }
    /**
     * @param {CellCoord} cell_coord
     */
    clear_cell_metal(cell_coord) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        wasm.buffer_clear_cell_metal(this.__wbg_ptr, ptr0);
    }
}

const CameraFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_camera_free(ptr >>> 0, 1));

export class Camera {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CameraFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_camera_free(ptr, 0);
    }
    /**
     * @returns {Vec2}
     */
    get translation() {
        const ret = wasm.__wbg_get_camera_translation(this.__wbg_ptr);
        return Vec2.__wrap(ret);
    }
    /**
     * @param {Vec2} arg0
     */
    set translation(arg0) {
        _assertClass(arg0, Vec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_camera_translation(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {number}
     */
    get scale() {
        const ret = wasm.__wbg_get_camera_scale(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set scale(arg0) {
        wasm.__wbg_set_camera_scale(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Vec2}
     */
    get size() {
        const ret = wasm.__wbg_get_camera_size(this.__wbg_ptr);
        return Vec2.__wrap(ret);
    }
    /**
     * @param {Vec2} arg0
     */
    set size(arg0) {
        _assertClass(arg0, Vec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_camera_size(this.__wbg_ptr, ptr0);
    }
    /**
     * @param {Vec2} translation
     * @param {number} scale
     */
    constructor(translation, scale) {
        _assertClass(translation, Vec2);
        var ptr0 = translation.__destroy_into_raw();
        const ret = wasm.camera_new_translation_scale(ptr0, scale);
        this.__wbg_ptr = ret >>> 0;
        CameraFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Project a screen x,y point into the world. Z axis is ignored because I don't need it.
     * @param {Vec2} position
     * @returns {Vec2}
     */
    project_screen_point_to_world(position) {
        _assertClass(position, Vec2);
        var ptr0 = position.__destroy_into_raw();
        const ret = wasm.camera_project_screen_point_to_world(this.__wbg_ptr, ptr0);
        return Vec2.__wrap(ret);
    }
    /**
     * Project a screen point to a cell location. It's the caller's responsibility to ensure the
     * point is within the visible bounds of the window.
     * @param {Vec2} position
     * @returns {CellCoord}
     */
    project_screen_point_to_cell(position) {
        _assertClass(position, Vec2);
        var ptr0 = position.__destroy_into_raw();
        const ret = wasm.camera_project_screen_point_to_cell(this.__wbg_ptr, ptr0);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} coord
     * @returns {Vec2}
     */
    project_cell_coord_to_screen_point(coord) {
        _assertClass(coord, CellCoord);
        var ptr0 = coord.__destroy_into_raw();
        const ret = wasm.camera_project_cell_coord_to_screen_point(this.__wbg_ptr, ptr0);
        return Vec2.__wrap(ret);
    }
}

const CellCoordFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_cellcoord_free(ptr >>> 0, 1));

export class CellCoord {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CellCoord.prototype);
        obj.__wbg_ptr = ptr;
        CellCoordFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CellCoordFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_cellcoord_free(ptr, 0);
    }
    /**
     * @returns {IVec2}
     */
    get 0() {
        const ret = wasm.__wbg_get_cellcoord_0(this.__wbg_ptr);
        return IVec2.__wrap(ret);
    }
    /**
     * @param {IVec2} arg0
     */
    set 0(arg0) {
        _assertClass(arg0, IVec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_cellcoord_0(this.__wbg_ptr, ptr0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.cellcoord__wasm_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        CellCoordFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const CompilerResultsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_compilerresults_free(ptr >>> 0, 1));

export class CompilerResults {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CompilerResultsFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_compilerresults_free(ptr, 0);
    }
    /**
     * @param {Buffer} buffer
     */
    constructor(buffer) {
        _assertClass(buffer, Buffer);
        const ret = wasm.compilerresults_from_buffer(buffer.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        CompilerResultsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Buffer} buffer
     * @param {Atom} edge_atom
     * @returns {(Atom)[]}
     */
    static get_trace_atoms(buffer, edge_atom) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(buffer, Buffer);
            _assertClass(edge_atom, Atom);
            var ptr0 = edge_atom.__destroy_into_raw();
            wasm.compilerresults_get_trace_atoms(retptr, buffer.__wbg_ptr, ptr0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v2 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_3(r0, r1 * 4, 4);
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const DMat2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dmat2_free(ptr >>> 0, 1));
/**
 * A 2x2 column major matrix.
 */
export class DMat2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DMat2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dmat2_free(ptr, 0);
    }
    /**
     * @returns {DVec2}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_dmat2_x_axis(this.__wbg_ptr);
        return DVec2.__wrap(ret);
    }
    /**
     * @param {DVec2} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, DVec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat2_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {DVec2}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_dmat2_y_axis(this.__wbg_ptr);
        return DVec2.__wrap(ret);
    }
    /**
     * @param {DVec2} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, DVec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat2_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @param {number} m00
     * @param {number} m01
     * @param {number} m10
     * @param {number} m11
     */
    constructor(m00, m01, m10, m11) {
        const ret = wasm.dmat2_wasm_bindgen_ctor(m00, m01, m10, m11);
        this.__wbg_ptr = ret >>> 0;
        DMat2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const DMat3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dmat3_free(ptr >>> 0, 1));
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

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DMat3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dmat3_free(ptr, 0);
    }
    /**
     * @returns {DVec3}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_dmat3_x_axis(this.__wbg_ptr);
        return DVec3.__wrap(ret);
    }
    /**
     * @param {DVec3} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, DVec3);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat3_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {DVec3}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_dmat3_y_axis(this.__wbg_ptr);
        return DVec3.__wrap(ret);
    }
    /**
     * @param {DVec3} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, DVec3);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat3_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {DVec3}
     */
    get z_axis() {
        const ret = wasm.__wbg_get_dmat3_z_axis(this.__wbg_ptr);
        return DVec3.__wrap(ret);
    }
    /**
     * @param {DVec3} arg0
     */
    set z_axis(arg0) {
        _assertClass(arg0, DVec3);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat3_z_axis(this.__wbg_ptr, ptr0);
    }
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
    constructor(m00, m01, m02, m10, m11, m12, m20, m21, m22) {
        const ret = wasm.dmat3_wasm_bindgen_ctor(m00, m01, m02, m10, m11, m12, m20, m21, m22);
        this.__wbg_ptr = ret >>> 0;
        DMat3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const DMat4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dmat4_free(ptr >>> 0, 1));
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

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DMat4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dmat4_free(ptr, 0);
    }
    /**
     * @returns {DVec4}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_dmat4_x_axis(this.__wbg_ptr);
        return DVec4.__wrap(ret);
    }
    /**
     * @param {DVec4} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, DVec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat4_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {DVec4}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_dmat4_y_axis(this.__wbg_ptr);
        return DVec4.__wrap(ret);
    }
    /**
     * @param {DVec4} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, DVec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat4_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {DVec4}
     */
    get z_axis() {
        const ret = wasm.__wbg_get_dmat4_z_axis(this.__wbg_ptr);
        return DVec4.__wrap(ret);
    }
    /**
     * @param {DVec4} arg0
     */
    set z_axis(arg0) {
        _assertClass(arg0, DVec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat4_z_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {DVec4}
     */
    get w_axis() {
        const ret = wasm.__wbg_get_dmat4_w_axis(this.__wbg_ptr);
        return DVec4.__wrap(ret);
    }
    /**
     * @param {DVec4} arg0
     */
    set w_axis(arg0) {
        _assertClass(arg0, DVec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_dmat4_w_axis(this.__wbg_ptr, ptr0);
    }
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
    constructor(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33) {
        const ret = wasm.dmat4_wasm_bindgen_ctor(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33);
        this.__wbg_ptr = ret >>> 0;
        DMat4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const DQuatFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dquat_free(ptr >>> 0, 1));
/**
 * A quaternion representing an orientation.
 *
 * This quaternion is intended to be of unit length but may denormalize due to
 * floating point "error creep" which can occur when successive quaternion
 * operations are applied.
 */
export class DQuat {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DQuatFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dquat_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_dquat_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_dquat_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_dquat_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_dquat_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_dquat_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_dquat_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_dquat_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_dquat_w(this.__wbg_ptr, arg0);
    }
}

const DVec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dvec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class DVec2 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DVec2.prototype);
        obj.__wbg_ptr = ptr;
        DVec2Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DVec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dvec2_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_dvec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_dvec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_dvec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_dvec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.dvec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        DVec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const DVec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dvec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class DVec3 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DVec3.prototype);
        obj.__wbg_ptr = ptr;
        DVec3Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DVec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dvec3_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_dquat_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_dquat_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_dquat_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_dquat_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_dquat_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_dquat_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.dvec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        DVec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const DVec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dvec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class DVec4 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DVec4.prototype);
        obj.__wbg_ptr = ptr;
        DVec4Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DVec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dvec4_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_dvec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_dvec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_dvec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_dvec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_dvec4_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_dvec4_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_dvec4_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_dvec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.dvec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        DVec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const DragFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_drag_free(ptr >>> 0, 1));

export class Drag {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Drag.prototype);
        obj.__wbg_ptr = ptr;
        DragFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DragFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_drag_free(ptr, 0);
    }
    /**
     * @returns {CellCoord}
     */
    get start() {
        const ret = wasm.__wbg_get_drag_start(this.__wbg_ptr);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} arg0
     */
    set start(arg0) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_drag_start(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {boolean}
     */
    get initial_impulse_vertical() {
        const ret = wasm.__wbg_get_drag_initial_impulse_vertical(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set initial_impulse_vertical(arg0) {
        wasm.__wbg_set_drag_initial_impulse_vertical(this.__wbg_ptr, arg0);
    }
}

const EditorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_editor_free(ptr >>> 0, 1));
/**
 * An Editor represents the underlying 'state' of an edit session, including the buffer data,
 * transient buffers, masks, tools, and active tool states. It can be thought of as an active
 * 'file'. It does not include anything having to do with the presentation of the editor, like
 * cameras, viewports, and so on.
 */
export class Editor {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EditorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_editor_free(ptr, 0);
    }
    /**
     * The active buffer that dispatched input will be rendered to (like drawing).
     * This is used as the base for rendering (with mouse-follow stacked on top of it).
     * @returns {Buffer}
     */
    get buffer() {
        const ret = wasm.__wbg_get_editor_buffer(this.__wbg_ptr);
        return Buffer.__wrap(ret);
    }
    /**
     * The active buffer that dispatched input will be rendered to (like drawing).
     * This is used as the base for rendering (with mouse-follow stacked on top of it).
     * @param {Buffer} arg0
     */
    set buffer(arg0) {
        _assertClass(arg0, Buffer);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_editor_buffer(this.__wbg_ptr, ptr0);
    }
    /**
     * The current render mask applied to the buffer.
     * @returns {Mask}
     */
    get mask() {
        const ret = wasm.__wbg_get_editor_mask(this.__wbg_ptr);
        return Mask.__wrap(ret);
    }
    /**
     * The current render mask applied to the buffer.
     * @param {Mask} arg0
     */
    set mask(arg0) {
        _assertClass(arg0, Mask);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_editor_mask(this.__wbg_ptr, ptr0);
    }
    /**
     * The selected (visual mode) cells
     * @returns {Selection}
     */
    get selection() {
        const ret = wasm.__wbg_get_editor_selection(this.__wbg_ptr);
        return Selection.__wrap(ret);
    }
    /**
     * The selected (visual mode) cells
     * @param {Selection} arg0
     */
    set selection(arg0) {
        _assertClass(arg0, Selection);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_editor_selection(this.__wbg_ptr, ptr0);
    }
    /**
     * The last used cursor location
     * @returns {CellCoord | undefined}
     */
    get cursor_coord() {
        const ret = wasm.__wbg_get_editor_cursor_coord(this.__wbg_ptr);
        return ret === 0 ? undefined : CellCoord.__wrap(ret);
    }
    /**
     * The last used cursor location
     * @param {CellCoord | undefined} [arg0]
     */
    set cursor_coord(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, CellCoord);
            ptr0 = arg0.__destroy_into_raw();
        }
        wasm.__wbg_set_editor_cursor_coord(this.__wbg_ptr, ptr0);
    }
    /**
     * The CSS style that should be applied to the cursor.
     * @returns {string}
     */
    get cursor_style() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_editor_cursor_style(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * The CSS style that should be applied to the cursor.
     * @param {string} arg0
     */
    set cursor_style(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_editor_cursor_style(this.__wbg_ptr, ptr0, len0);
    }
    constructor() {
        const ret = wasm.editor_new();
        this.__wbg_ptr = ret >>> 0;
        EditorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {IoState} io_state
     * @param {Camera} camera
     * @returns {EditorDispatchResult}
     */
    dispatch_event(io_state, camera) {
        _assertClass(io_state, IoState);
        _assertClass(camera, Camera);
        const ret = wasm.editor_dispatch_event(this.__wbg_ptr, io_state.__wbg_ptr, camera.__wbg_ptr);
        return EditorDispatchResult.__wrap(ret);
    }
}

const EditorDispatchResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_editordispatchresult_free(ptr >>> 0, 1));

export class EditorDispatchResult {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(EditorDispatchResult.prototype);
        obj.__wbg_ptr = ptr;
        EditorDispatchResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EditorDispatchResultFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_editordispatchresult_free(ptr, 0);
    }
    /**
     * @returns {Buffer | undefined}
     */
    get buffer_persist() {
        const ret = wasm.__wbg_get_editordispatchresult_buffer_persist(this.__wbg_ptr);
        return ret === 0 ? undefined : Buffer.__wrap(ret);
    }
    /**
     * @param {Buffer | undefined} [arg0]
     */
    set buffer_persist(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, Buffer);
            ptr0 = arg0.__destroy_into_raw();
        }
        wasm.__wbg_set_editordispatchresult_buffer_persist(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {(ToolPersist)[]}
     */
    get tools_persist() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_editordispatchresult_tools_persist(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_3(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {(ToolPersist)[]} arg0
     */
    set tools_persist(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_export_1);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_editordispatchresult_tools_persist(this.__wbg_ptr, ptr0, len0);
    }
}

const I16Vec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_i16vec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class I16Vec2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        I16Vec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_i16vec2_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_i16vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i16vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_i16vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i16vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.i16vec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        I16Vec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const I16Vec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_i16vec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class I16Vec3 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        I16Vec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_i16vec3_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_i16vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i16vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_i16vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i16vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_i16vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_i16vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.i16vec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        I16Vec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const I16Vec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_i16vec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class I16Vec4 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        I16Vec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_i16vec4_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_i16vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i16vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_i16vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i16vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_i16vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_i16vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_i16vec4_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_i16vec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.i16vec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        I16Vec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const I64Vec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_i64vec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class I64Vec2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        I64Vec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_i64vec2_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get x() {
        const ret = wasm.__wbg_get_i64vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i64vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get y() {
        const ret = wasm.__wbg_get_i64vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i64vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} x
     * @param {bigint} y
     */
    constructor(x, y) {
        const ret = wasm.i64vec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        I64Vec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const I64Vec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_i64vec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class I64Vec3 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        I64Vec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_i64vec3_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get x() {
        const ret = wasm.__wbg_get_i64vec3_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i64vec3_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get y() {
        const ret = wasm.__wbg_get_i64vec3_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i64vec3_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get z() {
        const ret = wasm.__wbg_get_i64vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_i64vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} x
     * @param {bigint} y
     * @param {bigint} z
     */
    constructor(x, y, z) {
        const ret = wasm.i64vec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        I64Vec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const I64Vec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_i64vec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class I64Vec4 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        I64Vec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_i64vec4_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get x() {
        const ret = wasm.__wbg_get_i64vec3_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i64vec3_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get y() {
        const ret = wasm.__wbg_get_i64vec3_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i64vec3_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get z() {
        const ret = wasm.__wbg_get_i64vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_i64vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get w() {
        const ret = wasm.__wbg_get_i64vec4_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_i64vec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} x
     * @param {bigint} y
     * @param {bigint} z
     * @param {bigint} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.i64vec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        I64Vec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const IVec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ivec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class IVec2 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(IVec2.prototype);
        obj.__wbg_ptr = ptr;
        IVec2Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IVec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ivec2_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_ivec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_ivec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_ivec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_ivec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.ivec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        IVec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const IVec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ivec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class IVec3 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IVec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ivec3_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_ivec3_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_ivec3_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_ivec3_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_ivec3_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_ivec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_ivec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.ivec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        IVec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const IVec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ivec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class IVec4 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IVec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ivec4_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_ivec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_ivec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_ivec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_ivec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_ivec4_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_ivec4_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_ivec4_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_ivec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.ivec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        IVec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const IoStateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_iostate_free(ptr >>> 0, 1));

export class IoState {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IoStateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_iostate_free(ptr, 0);
    }
    /**
     * @returns {BoolState}
     */
    get hovered() {
        const ret = wasm.__wbg_get_iostate_hovered(this.__wbg_ptr);
        return BoolState.__wrap(ret);
    }
    /**
     * @param {BoolState} arg0
     */
    set hovered(arg0) {
        _assertClass(arg0, BoolState);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_iostate_hovered(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {BoolState}
     */
    get primary() {
        const ret = wasm.__wbg_get_iostate_primary(this.__wbg_ptr);
        return BoolState.__wrap(ret);
    }
    /**
     * @param {BoolState} arg0
     */
    set primary(arg0) {
        _assertClass(arg0, BoolState);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_iostate_primary(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {BoolState}
     */
    get secondary() {
        const ret = wasm.__wbg_get_iostate_secondary(this.__wbg_ptr);
        return BoolState.__wrap(ret);
    }
    /**
     * @param {BoolState} arg0
     */
    set secondary(arg0) {
        _assertClass(arg0, BoolState);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_iostate_secondary(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Drag | undefined}
     */
    get drag() {
        const ret = wasm.__wbg_get_iostate_drag(this.__wbg_ptr);
        return ret === 0 ? undefined : Drag.__wrap(ret);
    }
    /**
     * @param {Drag | undefined} [arg0]
     */
    set drag(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, Drag);
            ptr0 = arg0.__destroy_into_raw();
        }
        wasm.__wbg_set_iostate_drag(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {(KeyState)[]}
     */
    get keys() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_iostate_keys(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_3(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {(KeyState)[]} arg0
     */
    set keys(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_export_1);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_iostate_keys(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Vec2}
     */
    get screen_point() {
        const ret = wasm.__wbg_get_iostate_screen_point(this.__wbg_ptr);
        return Vec2.__wrap(ret);
    }
    /**
     * @param {Vec2} arg0
     */
    set screen_point(arg0) {
        _assertClass(arg0, Vec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_iostate_screen_point(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {CellCoord}
     */
    get cell() {
        const ret = wasm.__wbg_get_iostate_cell(this.__wbg_ptr);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} arg0
     */
    set cell(arg0) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_iostate_cell(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {number}
     */
    get scroll_delta_y() {
        const ret = wasm.__wbg_get_iostate_scroll_delta_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set scroll_delta_y(arg0) {
        wasm.__wbg_set_iostate_scroll_delta_y(this.__wbg_ptr, arg0);
    }
    constructor() {
        const ret = wasm.iostate_new();
        this.__wbg_ptr = ret >>> 0;
        IoStateFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {KeyboardEvent} e
     */
    event_key_down(e) {
        wasm.iostate_event_key_down(this.__wbg_ptr, addHeapObject(e));
    }
    /**
     * @param {KeyboardEvent} e
     */
    event_key_up(e) {
        wasm.iostate_event_key_up(this.__wbg_ptr, addHeapObject(e));
    }
    /**
     * @param {MouseEvent} e
     * @param {Camera} camera
     */
    event_mouse(e, camera) {
        _assertClass(camera, Camera);
        wasm.iostate_event_mouse(this.__wbg_ptr, addHeapObject(e), camera.__wbg_ptr);
    }
    /**
     * @param {boolean} presence
     */
    event_mouse_presence(presence) {
        wasm.iostate_event_mouse_presence(this.__wbg_ptr, presence);
    }
    /**
     * @param {WheelEvent} e
     */
    event_wheel(e) {
        wasm.iostate_event_wheel(this.__wbg_ptr, addHeapObject(e));
    }
    /**
     * @param {string} key
     * @returns {BoolState}
     */
    get_key(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.iostate_get_key(this.__wbg_ptr, ptr0, len0);
        return BoolState.__wrap(ret);
    }
    /**
     * @param {string} key_code
     * @returns {BoolState}
     */
    get_key_code(key_code) {
        const ptr0 = passStringToWasm0(key_code, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.iostate_get_key_code(this.__wbg_ptr, ptr0, len0);
        return BoolState.__wrap(ret);
    }
    /**
     * @returns {(CellCoord)[]}
     */
    get_drag_path() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.iostate_get_drag_path(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_3(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const KeyStateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_keystate_free(ptr >>> 0, 1));

export class KeyState {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KeyState.prototype);
        obj.__wbg_ptr = ptr;
        KeyStateFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof KeyState)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KeyStateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_keystate_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get key_code() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_keystate_key_code(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set key_code(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_keystate_key_code(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get key() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_keystate_key(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set key(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_keystate_key(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {BoolState}
     */
    get state() {
        const ret = wasm.__wbg_get_keystate_state(this.__wbg_ptr);
        return BoolState.__wrap(ret);
    }
    /**
     * @param {BoolState} arg0
     */
    set state(arg0) {
        _assertClass(arg0, BoolState);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_keystate_state(this.__wbg_ptr, ptr0);
    }
}

const MaskFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mask_free(ptr >>> 0, 1));
/**
 * Much like a Buffer, except lacking any undo or transaction support. Designed to 'overlay' a
 * buffer, activating various atoms. Any active atom that does not overlay a cell is considered
 * undefined behavior.
 */
export class Mask {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Mask.prototype);
        obj.__wbg_ptr = ptr;
        MaskFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MaskFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mask_free(ptr, 0);
    }
}

const Mat2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mat2_free(ptr >>> 0, 1));
/**
 * A 2x2 column major matrix.
 */
export class Mat2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Mat2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mat2_free(ptr, 0);
    }
    /**
     * @returns {Vec2}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_mat2_x_axis(this.__wbg_ptr);
        return Vec2.__wrap(ret);
    }
    /**
     * @param {Vec2} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, Vec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat2_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec2}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_mat2_y_axis(this.__wbg_ptr);
        return Vec2.__wrap(ret);
    }
    /**
     * @param {Vec2} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, Vec2);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat2_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @param {number} m00
     * @param {number} m01
     * @param {number} m10
     * @param {number} m11
     */
    constructor(m00, m01, m10, m11) {
        const ret = wasm.mat2_wasm_bindgen_ctor(m00, m01, m10, m11);
        this.__wbg_ptr = ret >>> 0;
        Mat2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Mat3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mat3_free(ptr >>> 0, 1));
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

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Mat3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mat3_free(ptr, 0);
    }
    /**
     * @returns {Vec3}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_mat3_x_axis(this.__wbg_ptr);
        return Vec3.__wrap(ret);
    }
    /**
     * @param {Vec3} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, Vec3);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat3_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec3}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_mat3_y_axis(this.__wbg_ptr);
        return Vec3.__wrap(ret);
    }
    /**
     * @param {Vec3} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, Vec3);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat3_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec3}
     */
    get z_axis() {
        const ret = wasm.__wbg_get_mat3_z_axis(this.__wbg_ptr);
        return Vec3.__wrap(ret);
    }
    /**
     * @param {Vec3} arg0
     */
    set z_axis(arg0) {
        _assertClass(arg0, Vec3);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat3_z_axis(this.__wbg_ptr, ptr0);
    }
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
    constructor(m00, m01, m02, m10, m11, m12, m20, m21, m22) {
        const ret = wasm.mat3_wasm_bindgen_ctor(m00, m01, m02, m10, m11, m12, m20, m21, m22);
        this.__wbg_ptr = ret >>> 0;
        Mat3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Mat3AFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mat3a_free(ptr >>> 0, 1));
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

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Mat3AFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mat3a_free(ptr, 0);
    }
    /**
     * @returns {Vec3A}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_mat3a_x_axis(this.__wbg_ptr);
        return Vec3A.__wrap(ret);
    }
    /**
     * @param {Vec3A} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, Vec3A);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat3a_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec3A}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_mat3a_y_axis(this.__wbg_ptr);
        return Vec3A.__wrap(ret);
    }
    /**
     * @param {Vec3A} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, Vec3A);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat3a_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec3A}
     */
    get z_axis() {
        const ret = wasm.__wbg_get_mat3a_z_axis(this.__wbg_ptr);
        return Vec3A.__wrap(ret);
    }
    /**
     * @param {Vec3A} arg0
     */
    set z_axis(arg0) {
        _assertClass(arg0, Vec3A);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat3a_z_axis(this.__wbg_ptr, ptr0);
    }
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
    constructor(m00, m01, m02, m10, m11, m12, m20, m21, m22) {
        const ret = wasm.mat3a_wasm_bindgen_ctor(m00, m01, m02, m10, m11, m12, m20, m21, m22);
        this.__wbg_ptr = ret >>> 0;
        Mat3AFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Mat4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mat4_free(ptr >>> 0, 1));
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

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Mat4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mat4_free(ptr, 0);
    }
    /**
     * @returns {Vec4}
     */
    get x_axis() {
        const ret = wasm.__wbg_get_mat4_x_axis(this.__wbg_ptr);
        return Vec4.__wrap(ret);
    }
    /**
     * @param {Vec4} arg0
     */
    set x_axis(arg0) {
        _assertClass(arg0, Vec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat4_x_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec4}
     */
    get y_axis() {
        const ret = wasm.__wbg_get_mat4_y_axis(this.__wbg_ptr);
        return Vec4.__wrap(ret);
    }
    /**
     * @param {Vec4} arg0
     */
    set y_axis(arg0) {
        _assertClass(arg0, Vec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat4_y_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec4}
     */
    get z_axis() {
        const ret = wasm.__wbg_get_mat4_z_axis(this.__wbg_ptr);
        return Vec4.__wrap(ret);
    }
    /**
     * @param {Vec4} arg0
     */
    set z_axis(arg0) {
        _assertClass(arg0, Vec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat4_z_axis(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Vec4}
     */
    get w_axis() {
        const ret = wasm.__wbg_get_mat4_w_axis(this.__wbg_ptr);
        return Vec4.__wrap(ret);
    }
    /**
     * @param {Vec4} arg0
     */
    set w_axis(arg0) {
        _assertClass(arg0, Vec4);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_mat4_w_axis(this.__wbg_ptr, ptr0);
    }
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
    constructor(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33) {
        const ret = wasm.mat4_wasm_bindgen_ctor(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33);
        this.__wbg_ptr = ret >>> 0;
        Mat4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const NormalizedCellFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_normalizedcell_free(ptr >>> 0, 1));
/**
 * NormalizedCell exists purely as a programming convenience, especially for painting. When editing
 * cells it's easier to deal with the cell as a single struct, instead of as a collection of [0, 4]
 * Atoms. NormalizedCells should be treated as transient and not stored anywhere.
 */
export class NormalizedCell {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NormalizedCell.prototype);
        obj.__wbg_ptr = ptr;
        NormalizedCellFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NormalizedCellFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_normalizedcell_free(ptr, 0);
    }
    /**
     * @returns {Metal}
     */
    get metal() {
        const ret = wasm.__wbg_get_normalizedcell_metal(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @param {Metal} arg0
     */
    set metal(arg0) {
        wasm.__wbg_set_normalizedcell_metal(this.__wbg_ptr, addHeapObject(arg0));
    }
    /**
     * @returns {Silicon}
     */
    get si() {
        const ret = wasm.__wbg_get_normalizedcell_si(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @param {Silicon} arg0
     */
    set si(arg0) {
        wasm.__wbg_set_normalizedcell_si(this.__wbg_ptr, addHeapObject(arg0));
    }
}

const PinFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pin_free(ptr >>> 0, 1));

export class Pin {

    static __unwrap(jsValue) {
        if (!(jsValue instanceof Pin)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PinFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pin_free(ptr, 0);
    }
    /**
     * @returns {CellCoord}
     */
    get cell_coord() {
        const ret = wasm.__wbg_get_pin_cell_coord(this.__wbg_ptr);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} arg0
     */
    set cell_coord(arg0) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_pin_cell_coord(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {boolean}
     */
    get trigger() {
        const ret = wasm.__wbg_get_pin_trigger(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set trigger(arg0) {
        wasm.__wbg_set_pin_trigger(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get si_output_high() {
        const ret = wasm.__wbg_get_pin_si_output_high(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set si_output_high(arg0) {
        wasm.__wbg_set_pin_si_output_high(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get si_input_high() {
        const ret = wasm.__wbg_get_pin_si_input_high(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set si_input_high(arg0) {
        wasm.__wbg_set_pin_si_input_high(this.__wbg_ptr, arg0);
    }
    /**
     * @param {CellCoord} cell_coord
     * @param {boolean} trigger
     */
    constructor(cell_coord, trigger) {
        _assertClass(cell_coord, CellCoord);
        var ptr0 = cell_coord.__destroy_into_raw();
        const ret = wasm.pin_new(ptr0, trigger);
        this.__wbg_ptr = ret >>> 0;
        PinFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const PlacementFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_placement_free(ptr >>> 0, 1));
/**
 * Represents the various placements of Metal and Si within a Cell, including the 4 cardinal
 * directions, and the center "self" location (which is implicit when any cardinal direction is
 * set, but can also stand alone).
 */
export class Placement {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PlacementFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_placement_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    get up() {
        const ret = wasm.__wbg_get_placement_up(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set up(arg0) {
        wasm.__wbg_set_placement_up(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get right() {
        const ret = wasm.__wbg_get_placement_right(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set right(arg0) {
        wasm.__wbg_set_placement_right(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get down() {
        const ret = wasm.__wbg_get_placement_down(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set down(arg0) {
        wasm.__wbg_set_placement_down(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get left() {
        const ret = wasm.__wbg_get_placement_left(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set left(arg0) {
        wasm.__wbg_set_placement_left(this.__wbg_ptr, arg0);
    }
}

const QuatFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_quat_free(ptr >>> 0, 1));
/**
 * A quaternion representing an orientation.
 *
 * This quaternion is intended to be of unit length but may denormalize due to
 * floating point "error creep" which can occur when successive quaternion
 * operations are applied.
 */
export class Quat {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        QuatFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_quat_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_quat_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_quat_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_quat_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_quat_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_quat_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_quat_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_quat_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_quat_w(this.__wbg_ptr, arg0);
    }
}

const SelectionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_selection_free(ptr >>> 0, 1));

export class Selection {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Selection.prototype);
        obj.__wbg_ptr = ptr;
        SelectionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SelectionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_selection_free(ptr, 0);
    }
    /**
     * @returns {CellCoord}
     */
    get lower_left() {
        const ret = wasm.__wbg_get_selection_lower_left(this.__wbg_ptr);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} arg0
     */
    set lower_left(arg0) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_selection_lower_left(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {CellCoord}
     */
    get upper_right() {
        const ret = wasm.__wbg_get_selection_upper_right(this.__wbg_ptr);
        return CellCoord.__wrap(ret);
    }
    /**
     * @param {CellCoord} arg0
     */
    set upper_right(arg0) {
        _assertClass(arg0, CellCoord);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_selection_upper_right(this.__wbg_ptr, ptr0);
    }
}

const SocketFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_socket_free(ptr >>> 0, 1));

export class Socket {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SocketFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_socket_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    get always_update() {
        const ret = wasm.__wbg_get_socket_always_update(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set always_update(arg0) {
        wasm.__wbg_set_socket_always_update(this.__wbg_ptr, arg0);
    }
    /**
     * @param {(Pin)[]} pins
     * @param {boolean} always_update
     * @param {Function} update_callback
     */
    constructor(pins, always_update, update_callback) {
        const ptr0 = passArrayJsValueToWasm0(pins, wasm.__wbindgen_export_1);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.socket_new(ptr0, len0, always_update, addHeapObject(update_callback));
        this.__wbg_ptr = ret >>> 0;
        SocketFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const ToolPersistFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_toolpersist_free(ptr >>> 0, 1));

export class ToolPersist {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ToolPersist.prototype);
        obj.__wbg_ptr = ptr;
        ToolPersistFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof ToolPersist)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ToolPersistFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_toolpersist_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get tool_name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_toolpersist_tool_name(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set tool_name(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_toolpersist_tool_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Uint8Array}
     */
    get serialized_state() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_toolpersist_serialized_state(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_3(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {Uint8Array} arg0
     */
    set serialized_state(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_export_1);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_toolpersist_serialized_state(this.__wbg_ptr, ptr0, len0);
    }
}

const U16Vec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u16vec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class U16Vec2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U16Vec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u16vec2_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_u16vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_u16vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_u16vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_u16vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.u16vec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        U16Vec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const U16Vec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u16vec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class U16Vec3 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U16Vec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u16vec3_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_u16vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_u16vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_u16vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_u16vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_u16vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_u16vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.u16vec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        U16Vec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const U16Vec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u16vec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class U16Vec4 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U16Vec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u16vec4_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_u16vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_u16vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_u16vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_u16vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_u16vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_u16vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_u16vec4_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_u16vec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.u16vec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        U16Vec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const U64Vec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u64vec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class U64Vec2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U64Vec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u64vec2_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get x() {
        const ret = wasm.__wbg_get_u64vec2_x(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_u64vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get y() {
        const ret = wasm.__wbg_get_u64vec2_y(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_u64vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} x
     * @param {bigint} y
     */
    constructor(x, y) {
        const ret = wasm.u64vec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        U64Vec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const U64Vec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u64vec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class U64Vec3 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U64Vec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u64vec3_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get x() {
        const ret = wasm.__wbg_get_i64vec2_x(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_i64vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get y() {
        const ret = wasm.__wbg_get_i64vec2_y(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_i64vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get z() {
        const ret = wasm.__wbg_get_u64vec3_z(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_u64vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} x
     * @param {bigint} y
     * @param {bigint} z
     */
    constructor(x, y, z) {
        const ret = wasm.u64vec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        U64Vec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const U64Vec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u64vec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class U64Vec4 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U64Vec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u64vec4_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get x() {
        const ret = wasm.__wbg_get_u64vec2_x(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_u64vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get y() {
        const ret = wasm.__wbg_get_u64vec2_y(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_u64vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get z() {
        const ret = wasm.__wbg_get_u64vec4_z(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_u64vec4_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get w() {
        const ret = wasm.__wbg_get_u64vec4_w(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_u64vec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} x
     * @param {bigint} y
     * @param {bigint} z
     * @param {bigint} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.u64vec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        U64Vec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const UPCFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_upc_free(ptr >>> 0, 1));
/**
 * Universal Packed Cell format stores each cell as a bit packed [u8; 4], ready for direct blitting
 * to a GPU RGBu8 texture. Stored as [u8; 4] instead of u32 for endian agnosticism during blitting.
 * Does not encode BufferMask data. The first 16 bits are also encoded as part of Blueprint
 * serialization.
 */
export class UPC {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(UPC.prototype);
        obj.__wbg_ptr = ptr;
        UPCFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UPCFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_upc_free(ptr, 0);
    }
    /**
     * @returns {NormalizedCell}
     */
    normalize() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.upc_normalize(ptr);
        return NormalizedCell.__wrap(ret);
    }
    /**
     * @param {UPC} upc
     * @returns {UPC}
     */
    static denormalize(upc) {
        _assertClass(upc, UPC);
        var ptr0 = upc.__destroy_into_raw();
        const ret = wasm.upc_denormalize(ptr0);
        return UPC.__wrap(ret);
    }
}

const UVec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_uvec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class UVec2 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UVec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_uvec2_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_uvec2_x(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_uvec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_uvec2_y(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_uvec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.uvec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        UVec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const UVec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_uvec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class UVec3 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UVec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_uvec3_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_uvec3_x(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_uvec3_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_uvec3_y(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_uvec3_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_uvec3_z(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_uvec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.uvec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        UVec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const UVec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_uvec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class UVec4 {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UVec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_uvec4_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_uvec3_x(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_uvec3_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_uvec3_y(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_uvec3_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_uvec3_z(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_uvec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_uvec4_w(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_uvec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.uvec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        UVec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Vec2Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_vec2_free(ptr >>> 0, 1));
/**
 * A 2-dimensional vector.
 */
export class Vec2 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Vec2.prototype);
        obj.__wbg_ptr = ptr;
        Vec2Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Vec2Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_vec2_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_vec2_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_vec2_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_vec2_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_vec2_y(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    constructor(x, y) {
        const ret = wasm.vec2_wasm_bindgen_ctor(x, y);
        this.__wbg_ptr = ret >>> 0;
        Vec2Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Vec3Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_vec3_free(ptr >>> 0, 1));
/**
 * A 3-dimensional vector.
 */
export class Vec3 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Vec3.prototype);
        obj.__wbg_ptr = ptr;
        Vec3Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Vec3Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_vec3_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_vec3_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_vec3_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_vec3_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_vec3_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_vec3_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_vec3_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.vec3_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        Vec3Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Vec3AFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_vec3a_free(ptr >>> 0, 1));
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

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Vec3A.prototype);
        obj.__wbg_ptr = ptr;
        Vec3AFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Vec3AFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_vec3a_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_vec3a_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_vec3a_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_vec3a_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_vec3a_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_vec3a_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_vec3a_z(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    constructor(x, y, z) {
        const ret = wasm.vec3a_wasm_bindgen_ctor(x, y, z);
        this.__wbg_ptr = ret >>> 0;
        Vec3AFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const Vec4Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_vec4_free(ptr >>> 0, 1));
/**
 * A 4-dimensional vector.
 */
export class Vec4 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Vec4.prototype);
        obj.__wbg_ptr = ptr;
        Vec4Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Vec4Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_vec4_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_vec3a_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_vec3a_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_vec3a_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_vec3a_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_vec3a_z(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_vec3a_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.__wbg_get_vec4_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set w(arg0) {
        wasm.__wbg_set_vec4_w(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    constructor(x, y, z, w) {
        const ret = wasm.vec4_wasm_bindgen_ctor(x, y, z, w);
        this.__wbg_ptr = ret >>> 0;
        Vec4Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const ViewportFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_viewport_free(ptr >>> 0, 1));
/**
 * Represents only the presentation state of an on or off screen viewport for rendering.
 */
export class Viewport {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ViewportFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_viewport_free(ptr, 0);
    }
    /**
     * @param {HTMLCanvasElement} canvas
     */
    constructor(canvas) {
        const ret = wasm.viewport_new(addHeapObject(canvas));
        this.__wbg_ptr = ret >>> 0;
        ViewportFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Camera} camera
     * @param {Editor} editor
     */
    draw(camera, editor) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(camera, Camera);
            _assertClass(editor, Editor);
            wasm.viewport_draw(retptr, this.__wbg_ptr, camera.__wbg_ptr, editor.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_cellcoord_new = function(arg0) {
        const ret = CellCoord.__wrap(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_atom_new = function(arg0) {
        const ret = Atom.__wrap(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_toolpersist_new = function(arg0) {
        const ret = ToolPersist.__wrap(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_keystate_new = function(arg0) {
        const ret = KeyState.__wrap(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_pin_unwrap = function(arg0) {
        const ret = Pin.__unwrap(takeObject(arg0));
        return ret;
    };
    imports.wbg.__wbg_toolpersist_unwrap = function(arg0) {
        const ret = ToolPersist.__unwrap(takeObject(arg0));
        return ret;
    };
    imports.wbg.__wbg_keystate_unwrap = function(arg0) {
        const ret = KeyState.__unwrap(takeObject(arg0));
        return ret;
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = getObject(arg0);
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
        const ret = new Error();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function(arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_instanceof_WebGl2RenderingContext_8dbe5170d8fdea28 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof WebGL2RenderingContext;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_bindVertexArray_9971ca458d8940ea = function(arg0, arg1) {
        getObject(arg0).bindVertexArray(getObject(arg1));
    };
    imports.wbg.__wbg_bufferData_97b16c4aedab785a = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
    };
    imports.wbg.__wbg_createVertexArray_ec08b54b9f8c74ea = function(arg0) {
        const ret = getObject(arg0).createVertexArray();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_deleteVertexArray_112dd9bcd72ec608 = function(arg0, arg1) {
        getObject(arg0).deleteVertexArray(getObject(arg1));
    };
    imports.wbg.__wbg_texImage2D_8fdaf5862d8d4be3 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9 === 0 ? undefined : getArrayU8FromWasm0(arg9, arg10));
    }, arguments) };
    imports.wbg.__wbg_uniformMatrix4fv_5bf1d4fcb9b38046 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_activeTexture_a2e9931456fe92b4 = function(arg0, arg1) {
        getObject(arg0).activeTexture(arg1 >>> 0);
    };
    imports.wbg.__wbg_attachShader_299671ccaa78592c = function(arg0, arg1, arg2) {
        getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
    };
    imports.wbg.__wbg_bindBuffer_70e5a7ef4920142a = function(arg0, arg1, arg2) {
        getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_bindTexture_78210066cfdda8ac = function(arg0, arg1, arg2) {
        getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_compileShader_9680f4f1d833586c = function(arg0, arg1) {
        getObject(arg0).compileShader(getObject(arg1));
    };
    imports.wbg.__wbg_createBuffer_478457cb9beff1a3 = function(arg0) {
        const ret = getObject(arg0).createBuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createProgram_48b8a105fd0cfb35 = function(arg0) {
        const ret = getObject(arg0).createProgram();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createShader_f956a5ec67a77964 = function(arg0, arg1) {
        const ret = getObject(arg0).createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createTexture_3ebc81a77f42cd4b = function(arg0) {
        const ret = getObject(arg0).createTexture();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_deleteBuffer_4ab8b253a2ff7ec7 = function(arg0, arg1) {
        getObject(arg0).deleteBuffer(getObject(arg1));
    };
    imports.wbg.__wbg_deleteTexture_05e26b0508f0589d = function(arg0, arg1) {
        getObject(arg0).deleteTexture(getObject(arg1));
    };
    imports.wbg.__wbg_drawArrays_af53529e509d0c8b = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
    };
    imports.wbg.__wbg_enableVertexAttribArray_08b992ae13fe30a9 = function(arg0, arg1) {
        getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_getAttribLocation_c498bc242afbf700 = function(arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getAttribLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_getProgramInfoLog_16c69289b6a9c98e = function(arg0, arg1, arg2) {
        const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getProgramParameter_4c981ddc3b62dda8 = function(arg0, arg1, arg2) {
        const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getShaderInfoLog_afb2baaac4baaff5 = function(arg0, arg1, arg2) {
        const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getShaderParameter_e21fb00f8255b86b = function(arg0, arg1, arg2) {
        const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getUniformLocation_74149153bba4c4cb = function(arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_linkProgram_983c5972b815b0de = function(arg0, arg1) {
        getObject(arg0).linkProgram(getObject(arg1));
    };
    imports.wbg.__wbg_shaderSource_c36f18b5114855e7 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_texParameteri_a73df30f47a92fec = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_uniform1f_d2ba9f3d60c3859c = function(arg0, arg1, arg2) {
        getObject(arg0).uniform1f(getObject(arg1), arg2);
    };
    imports.wbg.__wbg_uniform1i_b7abcc7b3b4aee52 = function(arg0, arg1, arg2) {
        getObject(arg0).uniform1i(getObject(arg1), arg2);
    };
    imports.wbg.__wbg_uniform2i_4ec241fbb51f58de = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2i(getObject(arg1), arg2, arg3);
    };
    imports.wbg.__wbg_useProgram_8232847dbf97643a = function(arg0, arg1) {
        getObject(arg0).useProgram(getObject(arg1));
    };
    imports.wbg.__wbg_vertexAttribPointer_f602d22ecb0758f6 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    };
    imports.wbg.__wbg_viewport_e333f63662d91f3a = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).viewport(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_instanceof_Window_6575cd7f1322f82f = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_performance_8efa15a3e0d18099 = function(arg0) {
        const ret = getObject(arg0).performance;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_clientWidth_600f98ddd2b6cb36 = function(arg0) {
        const ret = getObject(arg0).clientWidth;
        return ret;
    };
    imports.wbg.__wbg_clientHeight_0f17075303285b38 = function(arg0) {
        const ret = getObject(arg0).clientHeight;
        return ret;
    };
    imports.wbg.__wbg_log_f740dc2253ea759b = function(arg0) {
        console.log(getObject(arg0));
    };
    imports.wbg.__wbg_warn_41503a1c2194de89 = function(arg0) {
        console.warn(getObject(arg0));
    };
    imports.wbg.__wbg_width_cd62a064492c4489 = function(arg0) {
        const ret = getObject(arg0).width;
        return ret;
    };
    imports.wbg.__wbg_setwidth_23bf2deedd907275 = function(arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    };
    imports.wbg.__wbg_height_f9f3ea69baf38ed4 = function(arg0) {
        const ret = getObject(arg0).height;
        return ret;
    };
    imports.wbg.__wbg_setheight_239dc283bbe50da4 = function(arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    };
    imports.wbg.__wbg_getContext_cfe4caa91ffe938e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2), getObject(arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_now_d3cbc9581625f686 = function(arg0) {
        const ret = getObject(arg0).now();
        return ret;
    };
    imports.wbg.__wbg_offsetX_79b2d23b78682ab7 = function(arg0) {
        const ret = getObject(arg0).offsetX;
        return ret;
    };
    imports.wbg.__wbg_offsetY_39cb724403a8302f = function(arg0) {
        const ret = getObject(arg0).offsetY;
        return ret;
    };
    imports.wbg.__wbg_buttons_2cb9e49b40e20105 = function(arg0) {
        const ret = getObject(arg0).buttons;
        return ret;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_key_001eb20ba3b3d2fd = function(arg0, arg1) {
        const ret = getObject(arg1).key;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_code_bec0d5222298000e = function(arg0, arg1) {
        const ret = getObject(arg1).code;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_deltaY_afd77a1b9e0d9ccd = function(arg0) {
        const ret = getObject(arg0).deltaY;
        return ret;
    };
    imports.wbg.__wbg_newnoargs_1ede4bf2ebbaaf43 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_a9ef466721e824f2 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_e69b5f66fda8f13c = function() {
        const ret = new Object();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_bf91bf94d9e04084 = function() { return handleError(function () {
        const ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_52dd9f07d03fd5f8 = function() { return handleError(function () {
        const ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_05c129bf37fcf1be = function() { return handleError(function () {
        const ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_3eca19bb09e9c484 = function() { return handleError(function () {
        const ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_buffer_ccaed51a635d8a2d = function(arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_fc445c2d308275d0 = function(arg0, arg1, arg2) {
        const ret = new Float32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_parse_51ee5409072379d3 = function() { return handleError(function (arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_stringify_eead5648c09faaf8 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(getObject(arg0));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return addHeapObject(ret);
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {
    imports.wbg.memory = memory || new WebAssembly.Memory({initial:19,maximum:16384,shared:true});
}

function __wbg_finalize_init(instance, module, thread_stack_size) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;

if (typeof thread_stack_size !== 'undefined' && (typeof thread_stack_size !== 'number' || thread_stack_size === 0 || thread_stack_size % 65536 !== 0)) { throw 'invalid stack size' }
wasm.__wbindgen_start(thread_stack_size);
return wasm;
}

function initSync(module, memory) {
    if (wasm !== undefined) return wasm;

    let thread_stack_size
    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module, memory, thread_stack_size} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports, memory);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module, thread_stack_size);
}

async function __wbg_init(module_or_path, memory) {
    if (wasm !== undefined) return wasm;

    let thread_stack_size
    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path, memory, thread_stack_size} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('logic_paint_rs_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports, memory);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module, thread_stack_size);
}

export { initSync };
export default __wbg_init;
