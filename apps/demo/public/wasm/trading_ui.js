let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

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
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

let WASM_VECTOR_LEN = 0;

const WasmChartFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmchart_free(ptr >>> 0, 1));

const WasmLempelZivComplexityFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmlempelzivcomplexity_free(ptr >>> 0, 1));

const WasmPermutationEntropyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmpermutationentropy_free(ptr >>> 0, 1));

const WasmShannonEntropyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmshannonentropy_free(ptr >>> 0, 1));

/**
 * Main WASM Chart instance that can be controlled from JavaScript
 */
export class WasmChart {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmChartFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmchart_free(ptr, 0);
    }
    /**
     * Add a single candle
     * @param {bigint} time
     * @param {number} o
     * @param {number} h
     * @param {number} l
     * @param {number} c
     * @param {number} v
     */
    addCandle(time, o, h, l, c, v) {
        wasm.wasmchart_addCandle(this.__wbg_ptr, time, o, h, l, c, v);
    }
    /**
     * Clear all tools
     */
    clearTools() {
        const ret = wasm.wasmchart_clearTools(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Fit viewport to data
     */
    fitToData() {
        wasm.wasmchart_fitToData(this.__wbg_ptr);
    }
    /**
     * Get all candles as JSON (for indicator calculations)
     * @returns {string}
     */
    getCandles() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getCandles(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Handle keyboard event
     * @param {string} key
     */
    onKeyDown(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmchart_onKeyDown(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Handle mouse up event
     * @param {number} x
     * @param {number} y
     * @param {number} button
     */
    onMouseUp(x, y, button) {
        wasm.wasmchart_onMouseUp(this.__wbg_ptr, x, y, button);
    }
    /**
     * Remove an indicator pane by pane ID.
     * @param {string} pane_id
     * @returns {boolean}
     */
    removePane(pane_id) {
        const ptr0 = passStringToWasm0(pane_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_removePane(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Remove a tool by ID
     * @param {string} id
     */
    removeTool(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_removeTool(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set candle data from JavaScript array
     * @param {string} candles_json
     */
    setCandles(candles_json) {
        const ptr0 = passStringToWasm0(candles_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setCandles(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Export chart state to JSON
     * @returns {string}
     */
    exportState() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmchart_exportState(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Import chart state from JSON
     * @param {string} json
     */
    importState(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_importState(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Query current log scale mode
     * @returns {boolean}
     */
    isLogScale() {
        const ret = wasm.wasmchart_isLogScale(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Handle touch end
     * @param {number} x
     * @param {number} y
     */
    onTouchEnd(x, y) {
        wasm.wasmchart_onTouchEnd(this.__wbg_ptr, x, y);
    }
    /**
     * Set trading session configurations from JSON array.
     * Each session: `{ name, open_utc: [h,m], close_utc: [h,m], color: [r,g,b,a], show_open, show_close }`
     * Pass an empty array `[]` to clear sessions.
     * Pass `"default"` as the string to load NYSE, London, Tokyo, Sydney presets.
     * @param {string} sessions_json
     */
    setSessions(sessions_json) {
        const ptr0 = passStringToWasm0(sessions_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setSessions(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set timezone offset in minutes from UTC.
     * Examples: 60 = UTC+1, -300 = UTC-5, 540 = UTC+9, 0 = UTC
     * @param {number} offset_minutes
     */
    setTimezone(offset_minutes) {
        wasm.wasmchart_setTimezone(this.__wbg_ptr, offset_minutes);
    }
    /**
     * Attach a canvas element for rendering
     * @param {HTMLCanvasElement} canvas
     */
    attachCanvas(canvas) {
        const ret = wasm.wasmchart_attachCanvas(this.__wbg_ptr, canvas);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Handle mouse down event
     * @param {number} x
     * @param {number} y
     * @param {number} button
     */
    onMouseDown(x, y, button) {
        wasm.wasmchart_onMouseDown(this.__wbg_ptr, x, y, button);
    }
    /**
     * Handle mouse move event
     * @param {number} x
     * @param {number} y
     */
    onMouseMove(x, y) {
        wasm.wasmchart_onMouseMove(this.__wbg_ptr, x, y);
    }
    /**
     * Handle touch move
     * @param {number} x
     * @param {number} y
     */
    onTouchMove(x, y) {
        wasm.wasmchart_onTouchMove(this.__wbg_ptr, x, y);
    }
    /**
     * Apply time scaling - user is dragging on time axis
     * @param {number} x
     */
    scaleTimeTo(x) {
        const ret = wasm.wasmchart_scaleTimeTo(this.__wbg_ptr, x);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Toggle logarithmic price scale
     * @param {boolean} enabled
     */
    setLogScale(enabled) {
        wasm.wasmchart_setLogScale(this.__wbg_ptr, enabled);
    }
    /**
     * Merge new candles into the existing dataset (delegates to CandleBuffer::append).
     *
     * Existing candles are kept; incoming candles are sorted and deduped.
     * Duplicate timestamps are overwritten by the incoming value.
     * @param {string} candles_json
     */
    appendCandles(candles_json) {
        const ptr0 = passStringToWasm0(candles_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_appendCandles(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create an ellipse drawing tool (bounding box defined by two corner points)
     * @param {string} id
     * @param {bigint} t1
     * @param {number} p1
     * @param {bigint} t2
     * @param {number} p2
     */
    createEllipse(id, t1, p1, t2, p2) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createEllipse(this.__wbg_ptr, ptr0, len0, t1, p1, t2, p2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * End time scaling - user released mouse
     */
    endTimeScale() {
        const ret = wasm.wasmchart_endTimeScale(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get current scale mode as string
     * @returns {string}
     */
    getScaleMode() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getScaleMode(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Handle mouse leave event
     */
    onMouseLeave() {
        wasm.wasmchart_onMouseLeave(this.__wbg_ptr);
    }
    /**
     * Handle mouse wheel event
     * @param {number} x
     * @param {number} y
     * @param {number} delta_y
     */
    onMouseWheel(x, y, delta_y) {
        wasm.wasmchart_onMouseWheel(this.__wbg_ptr, x, y, delta_y);
    }
    /**
     * Handle touch start
     * @param {number} x
     * @param {number} y
     */
    onTouchStart(x, y) {
        wasm.wasmchart_onTouchStart(this.__wbg_ptr, x, y);
    }
    /**
     * Apply price scaling - user is dragging on price axis
     * @param {number} y
     */
    scalePriceTo(y) {
        const ret = wasm.wasmchart_scalePriceTo(this.__wbg_ptr, y);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set price scale display mode.
     * `mode` must be one of: "price", "log", "percent", "indexed"
     * @param {string} mode
     */
    setScaleMode(mode) {
        const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setScaleMode(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * End price scaling - user released mouse
     */
    endPriceScale() {
        const ret = wasm.wasmchart_endPriceScale(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get current bar spacing extra value
     * @returns {number}
     */
    getBarSpacing() {
        const ret = wasm.wasmchart_getBarSpacing(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get current magnet mode as string
     * @returns {string}
     */
    getMagnetMode() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getMagnetMode(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Return pane layout as JSON with main + indicator fractions.
     * @returns {string}
     */
    getPaneLayout() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getPaneLayout(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Query current price axis lock state
     * @returns {boolean}
     */
    isPriceLocked() {
        const ret = wasm.wasmchart_isPriceLocked(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Handle double click event
     * @param {number} x
     * @param {number} y
     */
    onDoubleClick(x, y) {
        wasm.wasmchart_onDoubleClick(this.__wbg_ptr, x, y);
    }
    /**
     * Set additional bar spacing in CSS pixels (positive = wider bars, negative = narrower)
     * @param {number} extra_px
     */
    setBarSpacing(extra_px) {
        wasm.wasmchart_setBarSpacing(this.__wbg_ptr, extra_px);
    }
    /**
     * Set the magnet/snap mode for drawing tool placement.
     * `mode` must be one of: "off", "weak", "strong"
     * @param {string} mode
     */
    setMagnetMode(mode) {
        const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setMagnetMode(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a Fibonacci retracement drawing tool
     * @param {string} id
     * @param {bigint} t1
     * @param {number} p1
     * @param {bigint} t2
     * @param {number} p2
     */
    createFibonacci(id, t1, p1, t2, p2) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createFibonacci(this.__wbg_ptr, ptr0, len0, t1, p1, t2, p2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a rectangle drawing tool
     * @param {string} id
     * @param {bigint} t1
     * @param {number} p1
     * @param {bigint} t2
     * @param {number} p2
     */
    createRectangle(id, t1, p1, t2, p2) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createRectangle(this.__wbg_ptr, ptr0, len0, t1, p1, t2, p2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Reset time scale to fit all data (double-click)
     */
    resetTimeScale() {
        const ret = wasm.wasmchart_resetTimeScale(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set candle rendering style
     * @param {string} style
     */
    setCandleStyle(style) {
        const ptr0 = passStringToWasm0(style, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setCandleStyle(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Lock or unlock the price axis. When locked, fit_to_data() and reloading
     * candles will leave the price range unchanged.
     * @param {boolean} locked
     */
    setPriceLocked(locked) {
        wasm.wasmchart_setPriceLocked(this.__wbg_ptr, locked);
    }
    /**
     * Start time scaling - user clicked on time axis
     * @param {number} x
     */
    startTimeScale(x) {
        const ret = wasm.wasmchart_startTimeScale(this.__wbg_ptr, x);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a text label drawing tool
     * @param {string} id
     * @param {bigint} time
     * @param {number} price
     * @param {string} text
     */
    createTextLabel(id, time, price, text) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createTextLabel(this.__wbg_ptr, ptr0, len0, time, price, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a new trend line tool
     * @param {string} id
     * @param {bigint} start_time
     * @param {number} start_price
     * @param {bigint} end_time
     * @param {number} end_price
     */
    createTrendLine(id, start_time, start_price, end_time, end_price) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createTrendLine(this.__wbg_ptr, ptr0, len0, start_time, start_price, end_time, end_price);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get viewport info as JSON
     * @returns {any}
     */
    getViewportInfo() {
        const ret = wasm.wasmchart_getViewportInfo(this.__wbg_ptr);
        return ret;
    }
    /**
     * Reset price scale to auto-fit data (double-click)
     */
    resetPriceScale() {
        const ret = wasm.wasmchart_resetPriceScale(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Select drawing at canvas position. If additive is true, toggles membership.
     * @param {number} x
     * @param {number} y
     * @param {boolean} additive
     * @returns {boolean}
     */
    selectDrawingAt(x, y, additive) {
        const ret = wasm.wasmchart_selectDrawingAt(this.__wbg_ptr, x, y, additive);
        return ret !== 0;
    }
    /**
     * Replace all candles (delegates to CandleBuffer::snapshot).
     *
     * Backward-compatible alias for `setCandles`; both methods accept the
     * same JSON format.
     * @param {string} candles_json
     */
    setCandlesBatch(candles_json) {
        const ptr0 = passStringToWasm0(candles_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setCandlesBatch(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Show or hide session marker lines
     * @param {boolean} show
     */
    setShowSessions(show) {
        wasm.wasmchart_setShowSessions(this.__wbg_ptr, show);
    }
    /**
     * Start price scaling - user pressed mouse on price axis
     * @param {number} y
     */
    startPriceScale(y) {
        const ret = wasm.wasmchart_startPriceScale(this.__wbg_ptr, y);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Add or replace a comparison symbol rendered as normalized percent performance.
     * @param {string} symbol
     * @param {string} candles_json
     * @param {string} color
     */
    addCompareSymbol(symbol, candles_json, color) {
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(candles_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(color, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_addCompareSymbol(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create or replace an indicator pane. Returns the pane ID.
     * @param {string} indicator_id
     * @param {string} params_json
     * @returns {string}
     */
    addIndicatorPane(indicator_id, params_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(indicator_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(params_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.wasmchart_addIndicatorPane(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            deferred3_0 = ret[0];
            deferred3_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Get crosshair position as JSON
     * @returns {any}
     */
    getCrosshairInfo() {
        const ret = wasm.wasmchart_getCrosshairInfo(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get OHLC formatted string at crosshair
     * @returns {any}
     */
    getOHLCFormatted() {
        const ret = wasm.wasmchart_getOHLCFormatted(this.__wbg_ptr);
        return ret;
    }
    /**
     * Replace footprint candle data.
     * @param {string} candles_json
     */
    setFootprintData(candles_json) {
        const ptr0 = passStringToWasm0(candles_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_setFootprintData(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Return active comparison symbols as JSON.
     * @returns {string}
     */
    getCompareSymbols() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getCompareSymbols(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get current timezone offset in minutes
     * @returns {number}
     */
    getTimezoneOffset() {
        const ret = wasm.wasmchart_getTimezoneOffset(this.__wbg_ptr);
        return ret;
    }
    /**
     * Set bar width ratio (0.0 = auto, 0.1–0.95 = explicit fraction of slot)
     * @param {number} ratio
     */
    setBarWidthRatio(ratio) {
        wasm.wasmchart_setBarWidthRatio(this.__wbg_ptr, ratio);
    }
    /**
     * Snap a (time, price) coordinate to the nearest OHLC point when magnet is active.
     * Returns JSON: `{ time: i64, price: f64, snapped: bool }`
     * @param {bigint} time
     * @param {number} price
     * @returns {any}
     */
    snapToCandle(time, price) {
        const ret = wasm.wasmchart_snapToCandle(this.__wbg_ptr, time, price);
        return ret;
    }
    /**
     * Append or replace one footprint candle by timestamp.
     * @param {string} candle_json
     */
    addFootprintCandle(candle_json) {
        const ptr0 = passStringToWasm0(candle_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_addFootprintCandle(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a new vertical line tool
     * @param {string} id
     * @param {bigint} time
     */
    createVerticalLine(id, time) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createVerticalLine(this.__wbg_ptr, ptr0, len0, time);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set candle style, including Renko with brick size.
     * For renko: pass "renko" and provide brick_size > 0.
     * @param {number} brick_size
     */
    setRenkoBrickSize(brick_size) {
        const ret = wasm.wasmchart_setRenkoBrickSize(this.__wbg_ptr, brick_size);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Return selected drawing IDs as JSON.
     * @returns {string}
     */
    getSelectedDrawings() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getSelectedDrawings(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Remove a comparison symbol.
     * @param {string} symbol
     */
    removeCompareSymbol(symbol) {
        const ptr0 = passStringToWasm0(symbol, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmchart_removeCompareSymbol(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Enable or disable footprint rendering.
     * @param {boolean} enabled
     */
    setFootprintEnabled(enabled) {
        wasm.wasmchart_setFootprintEnabled(this.__wbg_ptr, enabled);
    }
    /**
     * Upsert a single candle by timestamp (delegates to CandleBuffer::update_running).
     *
     * Accepts a JSON object representing one candle.  If a candle with the
     * same `time` already exists it is replaced in-place; otherwise it is
     * inserted at the correct sorted position.
     * @param {string} candle_json
     */
    updateRunningCandle(candle_json) {
        const ptr0 = passStringToWasm0(candle_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_updateRunningCandle(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a new horizontal line tool
     * @param {string} id
     * @param {number} price
     */
    createHorizontalLine(id, price) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_createHorizontalLine(this.__wbg_ptr, ptr0, len0, price);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get candle at position (with hit-testing)
     * @param {number} x
     * @param {number} y
     * @returns {any}
     */
    getCandleAtPosition(x, y) {
        const ret = wasm.wasmchart_getCandleAtPosition(this.__wbg_ptr, x, y);
        return ret;
    }
    /**
     * Select all drawings whose nodes are fully inside a screen-space rectangle.
     * @param {number} x1
     * @param {number} y1
     * @param {number} x2
     * @param {number} y2
     * @param {boolean} additive
     * @returns {string}
     */
    selectDrawingsInRect(x1, y1, x2, y2, additive) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_selectDrawingsInRect(this.__wbg_ptr, x1, y1, x2, y2, additive);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Delete all selected drawings as one undoable operation.
     * @returns {number}
     */
    deleteSelectedDrawings() {
        const ret = wasm.wasmchart_deleteSelectedDrawings(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Set one pane height fraction, then normalize all panes.
     * @param {string} pane_id
     * @param {number} fraction
     */
    setPaneHeightFraction(pane_id, fraction) {
        const ptr0 = passStringToWasm0(pane_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmchart_setPaneHeightFraction(this.__wbg_ptr, ptr0, len0, fraction);
    }
    /**
     * Move all selected drawings to follow the current canvas position.
     * @param {number} x
     * @param {number} y
     */
    dragSelectedDrawingsTo(x, y) {
        wasm.wasmchart_dragSelectedDrawingsTo(this.__wbg_ptr, x, y);
    }
    /**
     * End a bulk drawing drag.
     */
    endSelectedDrawingsDrag() {
        wasm.wasmchart_endSelectedDrawingsDrag(this.__wbg_ptr);
    }
    /**
     * Start bulk-dragging selected drawings from a canvas position.
     * @param {number} x
     * @param {number} y
     * @returns {boolean}
     */
    startSelectedDrawingsDrag(x, y) {
        const ret = wasm.wasmchart_startSelectedDrawingsDrag(this.__wbg_ptr, x, y);
        return ret !== 0;
    }
    /**
     * Create a new chart instance
     * @param {number} width
     * @param {number} height
     * @param {string} timeframe
     */
    constructor(width, height, timeframe) {
        const ptr0 = passStringToWasm0(timeframe, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmchart_new(width, height, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmChartFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Redo the last undone drawing action. Returns true if there was something to redo.
     * @returns {boolean}
     */
    redo() {
        const ret = wasm.wasmchart_redo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Undo the last drawing action. Returns true if there was something to undo.
     * @returns {boolean}
     */
    undo() {
        const ret = wasm.wasmchart_undo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Render the chart
     */
    render() {
        const ret = wasm.wasmchart_render(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Resize the chart
     * @param {number} width
     * @param {number} height
     */
    resize(width, height) {
        const ret = wasm.wasmchart_resize(this.__wbg_ptr, width, height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Check if chart needs redraw
     * @returns {boolean}
     */
    isDirty() {
        const ret = wasm.wasmchart_isDirty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Get all tools as JSON
     * @returns {string}
     */
    getTools() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmchart_getTools(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Switch between dark and light theme
     * @param {boolean} dark
     */
    setTheme(dark) {
        wasm.wasmchart_setTheme(this.__wbg_ptr, dark);
    }
}
if (Symbol.dispose) WasmChart.prototype[Symbol.dispose] = WasmChart.prototype.free;

/**
 * Lempel-Ziv Complexity indicator (WASM wrapper)
 */
export class WasmLempelZivComplexity {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmLempelZivComplexityFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmlempelzivcomplexity_free(ptr, 0);
    }
    /**
     * Get current buffer length
     * @returns {number}
     */
    len() {
        const ret = wasm.wasmlempelzivcomplexity_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new Lempel-Ziv Complexity indicator
     *
     * # Arguments
     * * `period` - Window size (recommended: 50-200)
     * * `threshold` - Binary conversion threshold (0.0 = auto/median)
     * @param {number} period
     * @param {number} threshold
     */
    constructor(period, threshold) {
        const ret = wasm.wasmlempelzivcomplexity_new(period, threshold);
        this.__wbg_ptr = ret >>> 0;
        WasmLempelZivComplexityFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Calculate complexity for next value
     *
     * Returns normalized complexity [0, 1] or null if insufficient data
     * - High (> 0.7): Random, chaotic
     * - Medium (0.4-0.7): Normal
     * - Low (< 0.4): Structured, repeating patterns
     * @param {number} value
     * @returns {any}
     */
    next(value) {
        const ret = wasm.wasmlempelzivcomplexity_next(this.__wbg_ptr, value);
        return ret;
    }
    /**
     * Reset the indicator state
     */
    reset() {
        wasm.wasmlempelzivcomplexity_reset(this.__wbg_ptr);
    }
    /**
     * Calculate Lempel-Ziv Complexity for array of values
     *
     * Returns JSON array of complexity values
     * @param {Float64Array} values
     * @param {number} period
     * @param {number} threshold
     * @returns {string}
     */
    static calculate(values, period, threshold) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passArrayF64ToWasm0(values, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmlempelzivcomplexity_calculate(ptr0, len0, period, threshold);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}
if (Symbol.dispose) WasmLempelZivComplexity.prototype[Symbol.dispose] = WasmLempelZivComplexity.prototype.free;

/**
 * Permutation Entropy indicator (WASM wrapper)
 */
export class WasmPermutationEntropy {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmPermutationEntropyFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmpermutationentropy_free(ptr, 0);
    }
    /**
     * Get current buffer length
     * @returns {number}
     */
    len() {
        const ret = wasm.wasmpermutationentropy_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new Permutation Entropy indicator
     *
     * # Arguments
     * * `period` - Window size (recommended: 50-200)
     * * `embedding_dimension` - Pattern length (recommended: 3-5)
     * * `delay` - Time delay (recommended: 1)
     * @param {number} period
     * @param {number} embedding_dimension
     * @param {number} delay
     */
    constructor(period, embedding_dimension, delay) {
        const ret = wasm.wasmpermutationentropy_new(period, embedding_dimension, delay);
        this.__wbg_ptr = ret >>> 0;
        WasmPermutationEntropyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Calculate permutation entropy for next value
     *
     * Returns normalized entropy [0, 1] or null if insufficient data
     * - High (> 0.8): Random, unpredictable
     * - Medium (0.4-0.8): Normal
     * - Low (< 0.4): Strong ordinal patterns
     * @param {number} value
     * @returns {any}
     */
    next(value) {
        const ret = wasm.wasmpermutationentropy_next(this.__wbg_ptr, value);
        return ret;
    }
    /**
     * Reset the indicator state
     */
    reset() {
        wasm.wasmpermutationentropy_reset(this.__wbg_ptr);
    }
    /**
     * Calculate Permutation Entropy for array of values
     *
     * Returns JSON array of entropy values
     * @param {Float64Array} values
     * @param {number} period
     * @param {number} embedding_dimension
     * @param {number} delay
     * @returns {string}
     */
    static calculate(values, period, embedding_dimension, delay) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passArrayF64ToWasm0(values, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmpermutationentropy_calculate(ptr0, len0, period, embedding_dimension, delay);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}
if (Symbol.dispose) WasmPermutationEntropy.prototype[Symbol.dispose] = WasmPermutationEntropy.prototype.free;

/**
 * Shannon Entropy indicator (WASM wrapper)
 */
export class WasmShannonEntropy {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmShannonEntropyFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmshannonentropy_free(ptr, 0);
    }
    /**
     * Get current buffer length
     * @returns {number}
     */
    len() {
        const ret = wasm.wasmpermutationentropy_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new Shannon Entropy indicator
     *
     * # Arguments
     * * `period` - Window size (recommended: 14-50)
     * * `bins` - Number of histogram bins (recommended: 10-20)
     * @param {number} period
     * @param {number} bins
     */
    constructor(period, bins) {
        const ret = wasm.wasmshannonentropy_new(period, bins);
        this.__wbg_ptr = ret >>> 0;
        WasmShannonEntropyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Calculate entropy for next value
     *
     * Returns normalized entropy [0, 1] or null if insufficient data
     * - High (> 0.8): Random market
     * - Medium (0.4-0.8): Normal market
     * - Low (< 0.4): Structured market
     * @param {number} value
     * @returns {any}
     */
    next(value) {
        const ret = wasm.wasmshannonentropy_next(this.__wbg_ptr, value);
        return ret;
    }
    /**
     * Reset the indicator state
     */
    reset() {
        wasm.wasmpermutationentropy_reset(this.__wbg_ptr);
    }
    /**
     * Calculate Shannon Entropy for array of values
     *
     * Returns JSON array of entropy values
     * @param {Float64Array} values
     * @param {number} period
     * @param {number} bins
     * @returns {string}
     */
    static calculate(values, period, bins) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passArrayF64ToWasm0(values, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmshannonentropy_calculate(ptr0, len0, period, bins);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}
if (Symbol.dispose) WasmShannonEntropy.prototype[Symbol.dispose] = WasmShannonEntropy.prototype.free;

/**
 * Get all available indicator metadata as JSON
 * @returns {string}
 */
export function getAllIndicators() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.getAllIndicators();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Get specific indicator metadata by ID as JSON
 * @param {string} id
 * @returns {any}
 */
export function getIndicatorMetadata(id) {
    const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.getIndicatorMetadata(ptr0, len0);
    return ret;
}

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
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
    imports.wbg.__wbg___wbindgen_is_undefined_f6b95eab589e0269 = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_arc_c46ca66b5ec2f1ac = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.arc(arg1, arg2, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_beginPath_08eae248f93ea32d = function(arg0) {
        arg0.beginPath();
    };
    imports.wbg.__wbg_call_abb4ff46ce38be40 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clip_b0ea262c8f6089c3 = function(arg0) {
        arg0.clip();
    };
    imports.wbg.__wbg_closePath_86ede1f286898302 = function(arg0) {
        arg0.closePath();
    };
    imports.wbg.__wbg_devicePixelRatio_390dee26c70aa30f = function(arg0) {
        const ret = arg0.devicePixelRatio;
        return ret;
    };
    imports.wbg.__wbg_ellipse_8fe237473fd39db1 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        arg0.ellipse(arg1, arg2, arg3, arg4, arg5, arg6, arg7);
    }, arguments) };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_fillRect_84131220403e26a4 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.fillRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_fillText_56566d8049e84e17 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_fill_dd0f756eea36e037 = function(arg0) {
        arg0.fill();
    };
    imports.wbg.__wbg_getContext_01f42b234e833f0a = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getTime_ad1e9878a735af08 = function(arg0) {
        const ret = arg0.getTime();
        return ret;
    };
    imports.wbg.__wbg_height_a07787f693c253d2 = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_instanceof_CanvasRenderingContext2d_d070139aaac1459f = function(arg0) {
        let result;
        try {
            result = arg0 instanceof CanvasRenderingContext2D;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_b5cf7783caa68180 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_lineTo_4b884d8cebfc8c54 = function(arg0, arg1, arg2) {
        arg0.lineTo(arg1, arg2);
    };
    imports.wbg.__wbg_log_1d990106d99dacb7 = function(arg0) {
        console.log(arg0);
    };
    imports.wbg.__wbg_moveTo_36127921f1ca46a5 = function(arg0, arg1, arg2) {
        arg0.moveTo(arg1, arg2);
    };
    imports.wbg.__wbg_new_0_23cedd11d9b40c9d = function() {
        const ret = new Date();
        return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_new_no_args_cb138f77cf6151ee = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_rect_b19815cce9795d25 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.rect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_resetTransform_55764c071b6ddb99 = function() { return handleError(function (arg0) {
        arg0.resetTransform();
    }, arguments) };
    imports.wbg.__wbg_restore_6486cb1a7aa3af7b = function(arg0) {
        arg0.restore();
    };
    imports.wbg.__wbg_save_b8767cfd2ee7f600 = function(arg0) {
        arg0.save();
    };
    imports.wbg.__wbg_scale_ffe3f80756d323ae = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.scale(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_set_fillStyle_c9a0550307cd4671 = function(arg0, arg1, arg2) {
        arg0.fillStyle = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_font_37c5ab71d0189314 = function(arg0, arg1, arg2) {
        arg0.font = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_height_6f8f8ef4cb40e496 = function(arg0, arg1) {
        arg0.height = arg1 >>> 0;
    };
    imports.wbg.__wbg_set_lineCap_791e7648138cc371 = function(arg0, arg1, arg2) {
        arg0.lineCap = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_lineJoin_196c6ac02fd494c3 = function(arg0, arg1, arg2) {
        arg0.lineJoin = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_lineWidth_feda4b79a15c660b = function(arg0, arg1) {
        arg0.lineWidth = arg1;
    };
    imports.wbg.__wbg_set_strokeStyle_697a576d2d3fbeaa = function(arg0, arg1, arg2) {
        arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_textAlign_5d82eb01e9d2291e = function(arg0, arg1, arg2) {
        arg0.textAlign = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_textBaseline_9e8ed61033c5023d = function(arg0, arg1, arg2) {
        arg0.textBaseline = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_width_7ff7a22c6e9f423e = function(arg0, arg1) {
        arg0.width = arg1 >>> 0;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_769e6b65d6557335 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_60cf02db4de8e1c1 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_08f5a74c69739274 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_a8924b26aa92d024 = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_strokeRect_31a396bc4462b669 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.strokeRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_stroke_a18b81eb49ff370e = function(arg0) {
        arg0.stroke();
    };
    imports.wbg.__wbg_warn_6e567d0d926ff881 = function(arg0) {
        console.warn(arg0);
    };
    imports.wbg.__wbg_width_dd0cfe94d42f5143 = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
        // Cast intrinsic for `F64 -> Externref`.
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('trading_ui_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
