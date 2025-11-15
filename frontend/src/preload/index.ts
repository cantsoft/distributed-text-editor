import { contextBridge, ipcRenderer } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'

const { Doc } = require('../../native/index.node')

// Custom APIs for renderer
const api = {
  minimize: () => ipcRenderer.send('window:minimize'),
  maximize: () => ipcRenderer.send('window:maximize'),
  close: () => ipcRenderer.send('window:close'),
  createDocument: () => {
    const doc = new Doc()
    return {
      collectString: () => doc.collectString(),
      removeAbsolute: (pos: number) => doc.removeAbsolute(pos),
      insertAbsoluteWrapper: (pos: number, data: string) => doc.insertAbsoluteWrapper(pos, data)
    }
  }
}

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', electronAPI)
    contextBridge.exposeInMainWorld('api', api)
  } catch (error) {
    console.error(error)
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI
  // @ts-ignore (define in dts)
  window.api = api
}
