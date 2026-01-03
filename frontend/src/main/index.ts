import { app, shell, BrowserWindow, ipcMain } from 'electron';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import icon from '../../resources/icon.png?asset';

import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import * as path from "path";
import * as protobuf from "protobufjs";

let root: protobuf.Root | null = null;
let UserOperation: protobuf.Type | null = null;
let backend: ChildProcessWithoutNullStreams | null = null;

async function runBackendService() {
  let buffer = Buffer.alloc(0);

  root = await protobuf.load("../proto/frame.proto");
  UserOperation = root.lookupType("dte.UserOperation");
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
        const message = UserOperation!.decode(payload);
        console.log("[Node received form Rust]:", message);
      } catch (e) {
        console.error("Decode error:", e);
      }
    }
  });

  backend.stderr.on("data", (data) => {
    console.error(`[Rust Log]: ${data.toString()}`);
  });

  backend.on("close", (code) => {
    console.log(`Backend process exited with code ${code}`);
    process.exit(code ?? 0);
  });
}

async function onUserInput(data, position, type) {
  let message: protobuf.Message<{}> | null = null;
  try {
    switch (type) {
      case "insertLineBreak":
        data = "\n";
      // eslint-disable-next-line no-fallthrough
      case "insertText":
        console.log(data);
        message = UserOperation!.create({
          position: position,
          insert: { char: data.codePointAt(0) },
        });
      break;
      case "deleteContentBackward":
        message = UserOperation!.create({
          position: position,
          remove: {},
        });
      break;
      default:
        console.log("Unhandled user input event");
        return;
    }
    const payload = UserOperation?.encode(message!).finish();
    const header = Buffer.alloc(4);
    header.writeUInt32BE(payload!.length, 0);

    backend!.stdin.write(header);
    backend!.stdin.write(payload);
  } catch (e) { console.error("Encode/Send error:", e); }
}

function createWindow(): void {
  const main_window = new BrowserWindow({
    width: 800,
    minWidth: 400,
    height: 600,
    minHeight: 300,
    show: false,
    frame: false,
    ...(process.platform === 'linux' ? { icon } : {}),
    webPreferences: {
      preload: path.join(__dirname, '../preload/index.js'),
      sandbox: false
    }
  });

  ipcMain.on('window:close', () => { main_window.close(); });
  ipcMain.on('window:minimize', () => { main_window.minimize(); });
  ipcMain.on('window:maximize', () => {
    if (main_window.isMaximized()) { main_window.unmaximize(); }
    else { main_window.maximize(); }
  });
  ipcMain.on('user:input', (...args) => { 
    onUserInput(args[1], args[2], args[3]); 
  });

  main_window.on('ready-to-show', () => { main_window.show() });

  if (is.dev) {
    main_window.webContents.openDevTools();
  }

  main_window.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  });

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) { main_window.loadURL(process.env['ELECTRON_RENDERER_URL']); } 
  else { main_window.loadFile(path.join(__dirname, '../renderer/index.html')); }
}

app.whenReady().then(() => {

  // Set app user model id for windows
  electronApp.setAppUserModelId('com.electron');

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  // see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
  app.on('browser-window-created', (_, window) => { optimizer.watchWindowShortcuts(window); });

  runBackendService().catch((err) => console.error(err)); 
  createWindow();

  // On macOS it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) { createWindow(); }
  });
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') { app.quit(); }
});