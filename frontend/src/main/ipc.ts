import { BrowserWindow } from "electron";
import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import * as path from "path";
import * as protobuf from "protobufjs";

/**************************************************************************************************/

let root: protobuf.Root | null = null;
let LocalOpFrame: protobuf.Type | null = null;
let LocalCommandFrame: protobuf.Type | null = null;

let main_window: BrowserWindow | null = null
let backend: ChildProcessWithoutNullStreams | null = null;

interface LocalOp {
  position: number;
  in?: { value: number } | null;
  rm?: {} | null;
}

/**************************************************************************************************/

function handleMessage(message: LocalOp) {
  if (message.rm) {
    main_window!.webContents.send(
      "remove-request", 
      message.position!
    );
  } else if (message.in) {
    main_window!.webContents.send(
      "insert-request",
      message.position === undefined ? 0 : message.position,
      String.fromCharCode(message.in.value)
    );
  } else { console.error("Unknown message type"); }
}

/**************************************************************************************************/

function sendLocalCommand(message: protobuf.Message<{}>) {
  try {
    const payload = LocalCommandFrame!.encode(message!).finish();
    const header = Buffer.alloc(4);
    header.writeUInt32BE(payload!.length, 0);
    if (backend && backend.stdin) {
      backend.stdin.write(Buffer.concat([header, payload]));
    }
  } catch (e) { console.error("Encode/Send error:", e); }
}

/**************************************************************************************************/

export function updateBackendWindowReference(
  in_main_window: BrowserWindow
) { main_window = in_main_window; }

/**************************************************************************************************/

export async function runBackendService() {
  let buffer = Buffer.alloc(0);

  root = await protobuf.load("../proto/frames.proto");
  LocalOpFrame = root.lookupType("dte.LocalOp");
  LocalCommandFrame = root.lookupType("dte.LocalCommand");
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
        const message = LocalOpFrame!.decode(payload);
        handleMessage(LocalOpFrame!.toObject(message) as LocalOp);
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

export async function onKeyDown(key_data: string, cursor_pos: number): Promise<void> {
  let message: protobuf.Message<{}> | null = null;
  let data = key_data;
  switch (key_data) {
    case "Backspace":
      if (cursor_pos <= 0) { return; }
      message = LocalCommandFrame!.create({
        op: { position: cursor_pos, rm: {} }
      });
      break;
    case "Enter":
      data = "\n";
      message = LocalCommandFrame!.create({
        op: { position: cursor_pos, in: { value: data.codePointAt(0) } }
      });
      break;
    default:
      const code = key_data.codePointAt(0);
        message = LocalCommandFrame!.create({
          op: { position: cursor_pos, in: { value: code } }
        });
        break;
  }

  sendLocalCommand(message);

}

/**************************************************************************************************/

export function onSave() {
  sendLocalCommand(LocalCommandFrame!.create({ s: {} }));
}

/**************************************************************************************************/

export function onExit() {
  sendLocalCommand(LocalCommandFrame!.create({ c: {} }));
}

/**************************************************************************************************/