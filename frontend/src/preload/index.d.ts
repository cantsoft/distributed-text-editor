import { ElectronAPI } from '@electron-toolkit/preload'

const backend_api = require('../../native/index')

declare global {
  interface Window {
    electron: ElectronAPI
    api: {
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      createDocument: () => backend_api.Doc;
    } 
  }
}
