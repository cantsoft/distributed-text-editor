import { ElectronAPI } from '@electron-toolkit/preload'

const { Doc } = require('../../native/index.node')

declare global {
  interface Window {
    electron: ElectronAPI
    api: {
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      createDocument: () => any;
    }
  }
}
