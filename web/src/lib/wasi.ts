// A minimal WASI preview1 host, backed by a read-only in-memory filesystem.
//
// The engine is a WASI module because Swiss Ephemeris reads its data as files.
// A browser has no filesystem, so this provides one: the ephemeris, corpus and
// place index are fetched once and handed to the module as if they were on
// disk. Only the 17 calls the module actually imports are implemented, and
// only as far as it uses them — this is a host for one known guest, not a
// general WASI runtime.
//
// Specification: docs/phase12/DESIGN.md.

/** A directory tree of files. Keys are paths like "ephe/semo_18.se1". */
export type Files = Map<string, Uint8Array>;

// errno values from the preview1 specification.
const OK = 0;
const EBADF = 8;
const EINVAL = 28;
const EISDIR = 31;
const ENOENT = 44;
const ENOTDIR = 54;

const FILETYPE_DIRECTORY = 3;
const FILETYPE_REGULAR_FILE = 4;

interface Node {
  path: string;
  directory: boolean;
  data?: Uint8Array;
}

interface Handle {
  node: Node;
  offset: number;
}

export interface WasiOptions {
  /** Where the tree is mounted, as the guest sees it. */
  root: string;
  files: Files;
  /** Overridable so tests are deterministic. */
  now?: () => number;
  random?: (bytes: Uint8Array) => void;
  onStderr?: (text: string) => void;
}

export class Wasi {
  private view!: DataView;
  private bytes!: Uint8Array;
  private readonly handles = new Map<number, Handle>();
  private next = 4; // 0-2 are the standard streams, 3 is the preopened root
  private readonly dirs = new Set<string>();
  private readonly root: string;
  private readonly files: Files;
  private readonly now: () => number;
  private readonly random: (bytes: Uint8Array) => void;
  private readonly onStderr: (text: string) => void;

  constructor(options: WasiOptions) {
    this.root = options.root.replace(/\/+$/, "");
    this.files = options.files;
    this.now = options.now ?? (() => Date.now());
    this.random =
      options.random ??
      ((b) => {
        if (globalThis.crypto?.getRandomValues) globalThis.crypto.getRandomValues(b);
        else for (let i = 0; i < b.length; i++) b[i] = Math.floor(Math.random() * 256);
      });
    this.onStderr = options.onStderr ?? ((t) => console.error("[engine]", t));

    // Every ancestor directory of every file exists.
    this.dirs.add("");
    for (const path of this.files.keys()) {
      const parts = path.split("/");
      for (let i = 1; i < parts.length; i++) this.dirs.add(parts.slice(0, i).join("/"));
    }
    this.handles.set(3, { node: { path: "", directory: true }, offset: 0 });
  }

  /** Call once the module is instantiated, before any export. */
  bind(memory: WebAssembly.Memory): void {
    this.view = new DataView(memory.buffer);
    this.bytes = new Uint8Array(memory.buffer);
    // Growing memory detaches the buffer, so refresh the views on each call.
    this.refresh = () => {
      if (this.bytes.byteLength !== memory.buffer.byteLength) {
        this.view = new DataView(memory.buffer);
        this.bytes = new Uint8Array(memory.buffer);
      }
    };
  }

  private refresh: () => void = () => {};

  // -------------------------------------------------------------- helpers

  private str(ptr: number, len: number): string {
    this.refresh();
    return new TextDecoder().decode(this.bytes.subarray(ptr, ptr + len));
  }

  /** Resolve a guest path to a node, or null. Paths may be absolute. */
  private resolve(path: string): Node | null {
    let p = path;
    if (p.startsWith(this.root)) p = p.slice(this.root.length);
    p = p.replace(/^\/+/, "").replace(/\/+$/, "");
    // No "..": this host serves one fixed tree.
    if (p.split("/").includes("..")) return null;
    const data = this.files.get(p);
    if (data) return { path: p, directory: false, data };
    if (this.dirs.has(p)) return { path: p, directory: true };
    return null;
  }

  private entries(dir: string): Array<{ name: string; directory: boolean }> {
    const prefix = dir === "" ? "" : `${dir}/`;
    const seen = new Map<string, boolean>();
    for (const path of this.files.keys()) {
      if (!path.startsWith(prefix)) continue;
      const rest = path.slice(prefix.length);
      const slash = rest.indexOf("/");
      if (slash === -1) seen.set(rest, false);
      else seen.set(rest.slice(0, slash), true);
    }
    return [...seen].map(([name, directory]) => ({ name, directory }));
  }

  private filestat(ptr: number, node: Node): void {
    this.refresh();
    const time = BigInt(this.now()) * 1_000_000n;
    this.view.setBigUint64(ptr, 0n, true); // dev
    this.view.setBigUint64(ptr + 8, 0n, true); // ino
    this.view.setUint8(ptr + 16, node.directory ? FILETYPE_DIRECTORY : FILETYPE_REGULAR_FILE);
    this.view.setBigUint64(ptr + 24, 1n, true); // nlink
    this.view.setBigUint64(ptr + 32, BigInt(node.data?.length ?? 0), true); // size
    this.view.setBigUint64(ptr + 40, time, true); // atim
    this.view.setBigUint64(ptr + 48, time, true); // mtim
    this.view.setBigUint64(ptr + 56, time, true); // ctim
  }

  // ------------------------------------------------------- the 17 imports

  get imports(): WebAssembly.Imports {
    const self = this;
    return {
      wasi_snapshot_preview1: {
        fd_prestat_get(fd: number, ptr: number): number {
          if (fd !== 3) return EBADF;
          self.refresh();
          self.view.setUint8(ptr, 0); // a preopened directory
          self.view.setUint32(ptr + 4, new TextEncoder().encode(self.root).length, true);
          return OK;
        },

        fd_prestat_dir_name(fd: number, ptr: number, len: number): number {
          if (fd !== 3) return EBADF;
          self.refresh();
          const name = new TextEncoder().encode(self.root);
          self.bytes.set(name.subarray(0, len), ptr);
          return OK;
        },

        path_open(
          dirfd: number, _dirflags: number, pathPtr: number, pathLen: number,
          oflags: number, _base: bigint, _inheriting: bigint, _fdflags: number, fdPtr: number,
        ): number {
          if (!self.handles.has(dirfd)) return EBADF;
          const node = self.resolve(self.str(pathPtr, pathLen));
          if (!node) return ENOENT;
          // This filesystem is read-only: creating or truncating is refused.
          if (oflags & 0b0001 || oflags & 0b1000) return EINVAL;
          if (oflags & 0b0010 && !node.directory) return ENOTDIR;
          const fd = self.next++;
          self.handles.set(fd, { node, offset: 0 });
          self.refresh();
          self.view.setUint32(fdPtr, fd, true);
          return OK;
        },

        fd_close(fd: number): number {
          return self.handles.delete(fd) ? OK : EBADF;
        },

        fd_read(fd: number, iovs: number, iovsLen: number, nreadPtr: number): number {
          const h = self.handles.get(fd);
          if (!h) return EBADF;
          if (h.node.directory) return EISDIR;
          const data = h.node.data!;
          self.refresh();
          let read = 0;
          for (let i = 0; i < iovsLen; i++) {
            const base = self.view.getUint32(iovs + i * 8, true);
            const len = self.view.getUint32(iovs + i * 8 + 4, true);
            const chunk = data.subarray(h.offset, h.offset + len);
            self.bytes.set(chunk, base);
            h.offset += chunk.length;
            read += chunk.length;
            if (chunk.length < len) break; // end of file
          }
          self.view.setUint32(nreadPtr, read, true);
          return OK;
        },

        fd_seek(fd: number, offset: bigint, whence: number, newPtr: number): number {
          const h = self.handles.get(fd);
          if (!h) return EBADF;
          const size = h.node.data?.length ?? 0;
          const from = whence === 0 ? 0 : whence === 1 ? h.offset : size;
          const next = from + Number(offset);
          if (next < 0) return EINVAL;
          h.offset = next;
          self.refresh();
          self.view.setBigUint64(newPtr, BigInt(next), true);
          return OK;
        },

        fd_fdstat_get(fd: number, ptr: number): number {
          const h = self.handles.get(fd);
          self.refresh();
          const filetype = !h ? FILETYPE_REGULAR_FILE : h.node.directory ? FILETYPE_DIRECTORY : FILETYPE_REGULAR_FILE;
          if (!h && fd > 2) return EBADF;
          self.view.setUint8(ptr, filetype);
          self.view.setUint16(ptr + 2, 0, true);
          self.view.setBigUint64(ptr + 8, ~0n, true); // rights: everything this host supports
          self.view.setBigUint64(ptr + 16, ~0n, true);
          return OK;
        },

        fd_fdstat_set_flags(): number {
          return OK; // nothing here is non-blocking or appending
        },

        fd_filestat_get(fd: number, ptr: number): number {
          const h = self.handles.get(fd);
          if (!h) return EBADF;
          self.filestat(ptr, h.node);
          return OK;
        },

        path_filestat_get(dirfd: number, _flags: number, pathPtr: number, pathLen: number, ptr: number): number {
          if (!self.handles.has(dirfd)) return EBADF;
          const node = self.resolve(self.str(pathPtr, pathLen));
          if (!node) return ENOENT;
          self.filestat(ptr, node);
          return OK;
        },

        fd_readdir(fd: number, buf: number, bufLen: number, cookie: bigint, usedPtr: number): number {
          const h = self.handles.get(fd);
          if (!h) return EBADF;
          if (!h.node.directory) return ENOTDIR;
          self.refresh();
          const items = self.entries(h.node.path);
          let offset = 0;
          let index = Number(cookie);
          const encoder = new TextEncoder();
          while (index < items.length) {
            const item = items[index]!;
            const name = encoder.encode(item.name);
            if (offset + 24 + name.length > bufLen) break;
            self.view.setBigUint64(buf + offset, BigInt(index + 1), true); // d_next
            self.view.setBigUint64(buf + offset + 8, BigInt(index + 1), true); // d_ino
            self.view.setUint32(buf + offset + 16, name.length, true);
            self.view.setUint8(buf + offset + 20, item.directory ? FILETYPE_DIRECTORY : FILETYPE_REGULAR_FILE);
            self.bytes.set(name, buf + offset + 24);
            offset += 24 + name.length;
            index++;
          }
          self.view.setUint32(usedPtr, offset, true);
          return OK;
        },

        fd_write(fd: number, iovs: number, iovsLen: number, writtenPtr: number): number {
          self.refresh();
          let written = 0;
          const parts: Uint8Array[] = [];
          for (let i = 0; i < iovsLen; i++) {
            const base = self.view.getUint32(iovs + i * 8, true);
            const len = self.view.getUint32(iovs + i * 8 + 4, true);
            parts.push(self.bytes.slice(base, base + len));
            written += len;
          }
          // Only the engine's panic output ever comes through here.
          if (fd === 1 || fd === 2) {
            const text = parts.map((p) => new TextDecoder().decode(p)).join("").trimEnd();
            if (text) self.onStderr(text);
          }
          self.view.setUint32(writtenPtr, written, true);
          return OK;
        },

        clock_time_get(_id: number, _precision: bigint, ptr: number): number {
          self.refresh();
          self.view.setBigUint64(ptr, BigInt(Math.round(self.now())) * 1_000_000n, true);
          return OK;
        },

        random_get(ptr: number, len: number): number {
          self.refresh();
          const b = new Uint8Array(len);
          self.random(b);
          self.bytes.set(b, ptr);
          return OK;
        },

        environ_sizes_get(countPtr: number, sizePtr: number): number {
          self.refresh();
          self.view.setUint32(countPtr, 0, true);
          self.view.setUint32(sizePtr, 0, true);
          return OK;
        },

        environ_get(): number {
          return OK;
        },

        proc_exit(code: number): never {
          throw new Error(`the engine called proc_exit(${code})`);
        },
      },
    };
  }
}
