// The web build's data folder kept in the browser's IndexedDB: one record per file (its bytes)
// or folder (null), keyed by its path. Loaded once before the app starts (store_load), then
// changed as the app writes (store_file, store_dir, store_delete). Used by web.rs.

const NAME = "varstotas";
const FILES = "files";
let db = null;

function open() {
  return new Promise((resolve, reject) => {
    const r = indexedDB.open(NAME, 1);
    r.onupgradeneeded = () => r.result.createObjectStore(FILES);
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}

// Every record as [path, bytes or null, path, bytes or null, ...]; empty when there is no
// storage (a private window may have none).
export async function store_load() {
  try {
    db = await open();
  } catch (e) {
    console.warn("no IndexedDB: the data folder lasts only until the page is closed", e);
    return [];
  }
  // ask the browser not to clear it when space runs short
  if (navigator.storage && navigator.storage.persist) {
    navigator.storage.persist().catch(() => {});
  }
  return new Promise((resolve) => {
    const out = [];
    const r = db.transaction(FILES, "readonly").objectStore(FILES).openCursor();
    r.onsuccess = () => {
      const c = r.result;
      if (c) {
        out.push(c.key, c.value);
        c.continue();
      } else {
        resolve(out);
      }
    };
    r.onerror = () => resolve(out);
  });
}

function put(path, value) {
  if (db) {
    db.transaction(FILES, "readwrite").objectStore(FILES).put(value, path);
  }
}

// data is a view into the wasm memory: copied here, before the call returns.
export function store_file(path, data) {
  put(path, data.slice());
}

export function store_dir(path) {
  put(path, null);
}

export function store_delete(path) {
  if (db) {
    db.transaction(FILES, "readwrite").objectStore(FILES).delete(path);
  }
}
