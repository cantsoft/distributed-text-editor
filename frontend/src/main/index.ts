import { app, shell, BrowserWindow, ipcMain } from 'electron';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import icon from '../../resources/icon.png?asset';
import { join } from 'path';
const { spawn } = require('node:child_process');
const path = require('path');

function createWindow(): void {

  const backend = spawn(path.resolve(__dirname, '../../native/backend'));
  backend.stdout.on('data', (data) => {
    const text = data.toString();
    process.stdout.write(`[RUST]: ${text}`);
    try {
      const json = JSON.parse(text);
      console.log("Zdekodowano:", json);
    } catch (e) {
    }
  });
  backend.stdin.write("HELLO FROM FRONTEND\n");
  backend.on('close', (code) => {
    console.log(`Backend zakończony z kodem: ${code}`);
  });

  const main_window = new BrowserWindow({
    width: 800,
    minWidth: 400,
    height: 600,
    minHeight: 300,
    show: false,
    frame: false,
    ...(process.platform === 'linux' ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false
    }
  });

  ipcMain.on('window:close', () => { main_window.close(); });
  ipcMain.on('window:minimize', () => { main_window.minimize(); });
  ipcMain.on('window:maximize', () => {
    if (main_window.isMaximized()) { main_window.unmaximize(); }
    else { main_window.maximize(); }
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
  else { main_window.loadFile(join(__dirname, '../renderer/index.html')); }
}

app.whenReady().then(() => {

  // Set app user model id for windows
  electronApp.setAppUserModelId('com.electron');

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  // see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
  app.on('browser-window-created', (_, window) => { optimizer.watchWindowShortcuts(window); });

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