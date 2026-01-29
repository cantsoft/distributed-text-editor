import { BrowserWindow } from "electron";
import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import * as path from "path";
import * as protobuf from "protobufjs";

/**************************************************************************************************/

let root: protobuf.Root | null = null;
let ServerEventFrame: protobuf.Type | null = null;
let ClientCommandFrame: protobuf.Type | null = null;

let main_window: BrowserWindow | null = null;
let backend: ChildProcessWithoutNullStreams | null = null;

/**************************************************************************************************/


interface ServerEvent {
  op?: LocalOp | null;
  state?: FullState | null;
}

interface FullState {
  content: Uint8Array;
}

interface LocalOp {
  position: number;
  remote: boolean;
  insert?: { value: number } | null;
  remove?: object | null;
}

/**************************************************************************************************/

function handleServerEvent(event: ServerEvent): void {
  if (event.state) {
    const text = new TextDecoder("utf-8").decode(event.state.content);
    main_window!.webContents.send("full-sync-request", text);
    return;
  }
  if (event.op) {
    const op = event.op;
    const pos = op.position ?? 0;
    const remote = op.remote ?? 0;
    if (op.remove) {
      main_window!.webContents.send("remove-request", pos, remote);
    } else if (op.insert) {
      main_window!.webContents.send(
        "insert-request",
        pos,
        String.fromCharCode(op.insert.value),
        remote,
      );
    }
    return;
  }

  console.error("Unknown ServerEvent variant received:", event);
}

/**************************************************************************************************/

function sendLocalCommand(message: protobuf.Message<object>): void {
  try {
    const payload = ClientCommandFrame!.encode(message!).finish();
    const header = Buffer.alloc(4);
    header.writeUInt32BE(payload!.length, 0);
    if (backend && backend.stdin) {
      backend.stdin.write(Buffer.concat([header, payload]));
    }
  } catch (e) { console.error("Encode/Send error:", e); }
}

/**************************************************************************************************/

export function updateBackendWindowReference(
  in_main_window: BrowserWindow,
): void {
  main_window = in_main_window;
}

/**************************************************************************************************/

export async function runBackendService(): Promise<void> {
  let buffer = Buffer.alloc(0);

  root = await protobuf.load("../proto/frames.proto");
  ServerEventFrame = root.lookupType("dte.ServerEvent");
  ClientCommandFrame = root.lookupType("dte.ClientCommand");
  backend = spawn(path.resolve(__dirname, "../../native/backend"));

  backend.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    while (true) {
      if (buffer.length < 4) break;
      const msgLen = buffer.readUInt32BE(0);
      if (buffer.length < 4 + msgLen) break;

      const payload = buffer.subarray(4, 4 + msgLen);
      buffer = buffer.subarray(4 + msgLen);

      try {
        const message = ServerEventFrame!.decode(payload);
        console.log(message);
        handleServerEvent(ServerEventFrame!.toObject(message) as ServerEvent);
      } catch (e) {
        console.error("Decode error:", e);
      }
    }
  });

  backend.stderr.on("data", (data) => {
    console.log(`[Rust Log]: ${data.toString()}`);
  });

  backend.on("close", (code) => {
    console.log(`Backend process exited with code ${code}`);
    process.exit(code ?? 0);
  });
}

/**************************************************************************************************/

export async function onKeyDown(
  key_data: string,
  cursor_pos: number,
): Promise<void> {
  let message: protobuf.Message<object> | null = null;
  let data = key_data;
  switch (key_data) {
    case "Backspace":
      if (cursor_pos <= 0) { return; }
      message = ClientCommandFrame!.create({
        edit: { position: cursor_pos, remove: {} },
      });
      break;
    case "Enter":
      data = "\n";
      message = ClientCommandFrame!.create({
        edit: { position: cursor_pos, insert: { value: data.codePointAt(0) } },
      });
      break;
    default: {
      const code = key_data.codePointAt(0);
      message = ClientCommandFrame!.create({
        edit: { position: cursor_pos, insert: { value: code } },
      });
      break;
    }
  }

  sendLocalCommand(message);
}

/**************************************************************************************************/

export function onSave(filename: string): void {
  sendLocalCommand(ClientCommandFrame!.create({ save: { filename: filename } }));
}

/**************************************************************************************************/

export function onExit(): void {
  sendLocalCommand(ClientCommandFrame!.create({ close: {} }));
}

/**************************************************************************************************/
