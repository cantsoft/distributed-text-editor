import { app, shell, BrowserWindow, ipcMain } from 'electron';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import icon from '../../resources/icon.png?asset';
import * as path from "path";

import { runBackendService, onKeyDown, updateBackendWindowReference, onExit } from './ipc'

let main_window: BrowserWindow | null = null;

/**************************************************************************************************/

function createWindow(): void {
  main_window = new BrowserWindow({
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

  ipcMain.on('window:close', () => { main_window!.close(); });
  ipcMain.on('window:minimize', () => { main_window!.minimize(); });
  ipcMain.on('window:maximize', () => {
    if (main_window!.isMaximized()) { main_window!.unmaximize(); }
    else { main_window!.maximize(); }
  });
  ipcMain.on("user:keydown", (_event: any, key_data: string, cursor_pos: number) => { onKeyDown(key_data, cursor_pos); });
  
  main_window.on('ready-to-show', () => { main_window!.show() });

  // if (is.dev) {
  //   main_window.webContents.openDevTools();
  // }

  main_window.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  });

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) { main_window.loadURL(process.env['ELECTRON_RENDERER_URL']); } 
  else { main_window.loadFile(path.join(__dirname, '../renderer/index.html')); }

  updateBackendWindowReference(main_window);
}

/**************************************************************************************************/

app.whenReady().then(() => {
  electronApp.setAppUserModelId('com.electron');

  app.on('browser-window-created', (_, window) => { optimizer.watchWindowShortcuts(window); });

  runBackendService().catch((err) => console.error(err)); 
  createWindow();

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) { createWindow(); }
  });
});

/**************************************************************************************************/

app.on('window-all-closed', async () => {
  onExit();
  if (process.platform !== 'darwin') { app.quit(); }
});

/**************************************************************************************************/