// Auto-generated WASM wrapper for pgls
// This provides an Emscripten-compatible interface for the WASM module

/**
 * Create and initialize the PGLS WASM module.
 * @param {Object} options - Configuration options
 * @param {string} options.wasmPath - Custom path to the WASM file
 * @returns {Promise<Object>} The initialized WASM module
 */
export async function createPGLS(options = {}) {
  const wasmPath = options.wasmPath || new URL('./pgls.wasm', import.meta.url).href;

  // Fetch and compile the WASM module
  const response = await fetch(wasmPath);
  const wasmBytes = await response.arrayBuffer();

  // Variables for the module
  let wasmExports = null;
  let wasmMemory = null;
  let wasmTable = null;

  // Exception handling state (Emscripten runtime)
  let thrownValue = null;
  let threwValue = 0;

  // Helper to call a function with exception handling
  function invoke(fn, ...args) {
    const sp = wasmExports.emscripten_stack_get_current();
    try {
      return fn(...args);
    } catch (e) {
      wasmExports._emscripten_stack_restore(sp);
      // Handle longjmp - this is normal error handling in libpg_query
      if (e === 'longjmp') {
        wasmExports.setThrew(1, 0);
        return 0;
      }
      // Numeric exceptions are Emscripten's C++ exception pointers
      if (e !== e+0) throw e;
      wasmExports.setThrew(1, 0);
      return 0;
    }
  }

  // Create import object for the WASM module with Emscripten shims
  const importObject = {
    env: {
      // Emscripten exception handling
      __cxa_throw: (ptr, type, destructor) => {
        thrownValue = ptr;
        throw ptr;
      },
      __cxa_begin_catch: (ptr) => ptr,
      __cxa_find_matching_catch_2: () => {
        wasmExports.setThrew(thrownValue, 0);
        return thrownValue;
      },
      __cxa_find_matching_catch_3: () => {
        wasmExports.setThrew(thrownValue, 0);
        return thrownValue;
      },
      __resumeException: (ptr) => { throw ptr; },
      _emscripten_throw_longjmp: () => { throw 'longjmp'; },

      // Invoke wrappers for exception handling
      invoke_i: (index) => invoke(() => wasmTable.get(index)()),
      invoke_ii: (index, a1) => invoke(() => wasmTable.get(index)(a1)),
      invoke_iii: (index, a1, a2) => invoke(() => wasmTable.get(index)(a1, a2)),
      invoke_iiii: (index, a1, a2, a3) => invoke(() => wasmTable.get(index)(a1, a2, a3)),
      invoke_iiiii: (index, a1, a2, a3, a4) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4)),
      invoke_iiiiii: (index, a1, a2, a3, a4, a5) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4, a5)),
      invoke_v: (index) => invoke(() => wasmTable.get(index)()),
      invoke_vi: (index, a1) => invoke(() => wasmTable.get(index)(a1)),
      invoke_vii: (index, a1, a2) => invoke(() => wasmTable.get(index)(a1, a2)),
      invoke_viii: (index, a1, a2, a3) => invoke(() => wasmTable.get(index)(a1, a2, a3)),
      invoke_viiii: (index, a1, a2, a3, a4) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4)),
      invoke_viiiii: (index, a1, a2, a3, a4, a5) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4, a5)),
      invoke_viiiiii: (index, a1, a2, a3, a4, a5, a6) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4, a5, a6)),
      invoke_jii: (index, a1, a2) => invoke(() => wasmTable.get(index)(a1, a2)),
      invoke_iiji: (index, a1, a2, a3) => invoke(() => wasmTable.get(index)(a1, a2, a3)),
      invoke_vijiji: (index, a1, a2, a3, a4, a5) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4, a5)),
      invoke_viijii: (index, a1, a2, a3, a4, a5) => invoke(() => wasmTable.get(index)(a1, a2, a3, a4, a5)),

      // System calls
      __main_argc_argv: () => 0,
      __syscall_getcwd: (buf, size) => {
        const cwd = '/';
        const encoder = new TextEncoder();
        const bytes = encoder.encode(cwd + '\0');
        new Uint8Array(wasmMemory.buffer, buf, size).set(bytes.slice(0, size));
        return bytes.length;
      },
    },
    wasi_snapshot_preview1: {
      args_sizes_get: (pArgc, pArgvBufSize) => {
        new DataView(wasmMemory.buffer).setUint32(pArgc, 0, true);
        new DataView(wasmMemory.buffer).setUint32(pArgvBufSize, 0, true);
        return 0;
      },
      args_get: () => 0,
      environ_sizes_get: (pCount, pSize) => {
        new DataView(wasmMemory.buffer).setUint32(pCount, 0, true);
        new DataView(wasmMemory.buffer).setUint32(pSize, 0, true);
        return 0;
      },
      environ_get: () => 0,
      proc_exit: (code) => { throw { __procExit: true, code }; },
      fd_close: () => 0,
      fd_read: () => 0,
      fd_write: (fd, iovs, iovsLen, pNwritten) => {
        let written = 0;
        const view = new DataView(wasmMemory.buffer);
        const mem = new Uint8Array(wasmMemory.buffer);
        for (let i = 0; i < iovsLen; i++) {
          const ptr = view.getUint32(iovs + i * 8, true);
          const len = view.getUint32(iovs + i * 8 + 4, true);
          const str = new TextDecoder().decode(mem.slice(ptr, ptr + len));
          if (fd === 1) console.log(str);
          else if (fd === 2) console.error(str);
          written += len;
        }
        view.setUint32(pNwritten, written, true);
        return 0;
      },
      fd_seek: () => 0,
      clock_time_get: (clockId, precision, pTime) => {
        const time = BigInt(Date.now()) * 1000000n;
        new DataView(wasmMemory.buffer).setBigUint64(pTime, time, true);
        return 0;
      },
    },
  };

  const { instance } = await WebAssembly.instantiate(wasmBytes, importObject);
  wasmExports = instance.exports;
  wasmMemory = wasmExports.memory;
  wasmTable = wasmExports.__indirect_function_table;

  // Call _start if it exists (Emscripten initialization)
  // Note: _start may call proc_exit(0) which we catch and ignore
  if (wasmExports._start) {
    try {
      wasmExports._start();
    } catch (e) {
      // Handle proc_exit - only error on non-zero exit codes
      if (e && e.__procExit) {
        if (e.code !== 0) {
          throw new Error(`Process exited with code ${e.code}`);
        }
        // Exit code 0 is success, continue
      } else if (e === 'longjmp') {
        // Longjmp, continue
      } else {
        throw e;
      }
    }
  }

  // Emscripten-compatible runtime methods
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  /**
   * Calculate the byte length of a UTF-8 string.
   */
  function lengthBytesUTF8(str) {
    return encoder.encode(str).length;
  }

  /**
   * Write a string to WASM memory at the given pointer.
   */
  function stringToUTF8(str, ptr, maxLength) {
    const bytes = encoder.encode(str);
    const view = new Uint8Array(wasmMemory.buffer, ptr, maxLength);
    const len = Math.min(bytes.length, maxLength - 1);
    view.set(bytes.subarray(0, len));
    view[len] = 0; // Null terminator
  }

  /**
   * Read a null-terminated string from WASM memory.
   */
  function UTF8ToString(ptr) {
    if (ptr === 0) return '';
    const view = new Uint8Array(wasmMemory.buffer);
    let end = ptr;
    while (view[end] !== 0) end++;
    return decoder.decode(view.subarray(ptr, end));
  }

  // Return the module interface matching PGLSModule type
  return {
    // Memory management
    _malloc: wasmExports.malloc,
    _free: wasmExports.free,

    // FFI functions (prefixed with _ for compatibility)
    _pgls_init: wasmExports.pgls_init,
    _pgls_free_string: wasmExports.pgls_free_string,
    _pgls_set_schema: wasmExports.pgls_set_schema,
    _pgls_clear_schema: wasmExports.pgls_clear_schema,
    _pgls_insert_file: wasmExports.pgls_insert_file,
    _pgls_remove_file: wasmExports.pgls_remove_file,
    _pgls_lint: wasmExports.pgls_lint,
    _pgls_complete: wasmExports.pgls_complete,
    _pgls_hover: wasmExports.pgls_hover,
    _pgls_parse: wasmExports.pgls_parse,
    _pgls_version: wasmExports.pgls_version,

    // Emscripten runtime methods
    UTF8ToString,
    stringToUTF8,
    lengthBytesUTF8,

    // Direct access to exports and memory
    _exports: wasmExports,
    memory: wasmMemory,
  };
}

export default createPGLS;
