import { contextBridge, ipcRenderer } from "electron";
import { electronAPI } from "@electron-toolkit/preload";

// Custom APIs for renderer
const api = {
  minimize: () => ipcRenderer.send("window:minimize"),
  maximize: () => ipcRenderer.send("window:maximize"),
  close: () => ipcRenderer.send("window:close"),
  save: (filename: string) => ipcRenderer.send("user:save", filename),
  onUserKeydown: (keyData, cursorPos) => ipcRenderer.send("user:keydown", keyData, cursorPos),
  onRemoveRequest: (
    callback: (position: number, is_remote: boolean) => void,
  ) => {
    ipcRenderer.on("remove-request", (_e, position: number, is_remote: boolean) =>
        callback(position, is_remote),
    );
  },
  onInsertRequest: (
    callback: (position: number, char: string, is_remote: boolean) => void,
  ) => {
    ipcRenderer.on("insert-request", (_e, position: number, char: string, is_remote: boolean) =>
      callback(position, char, is_remote),
    );
  },
  onFullSync: (callback: (new_text: string) => void) => {
    ipcRenderer.on("full-sync-request", (_e, new_text: string) =>
      callback(new_text),
    );
  },
};

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld("electron", electronAPI);
    contextBridge.exposeInMainWorld("api", api);
  } catch (error) {
    console.error(error);
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI;
  // @ts-ignore (define in dts)
  window.api = api;
}
