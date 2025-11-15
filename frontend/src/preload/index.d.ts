import { ElectronAPI } from "@electron-toolkit/preload";
import { Doc } from "../../native";

declare global {
  interface Window {
    electron: ElectronAPI
    api: {
      minimize: () => void
      maximize: () => void
      close: () => void
      createDocument: () => {
        collectString: () => string
        removeAbsolute: (pos: number) => void
        insertAbsoluteWrapper: (pos: number, data: string) => void
      }
    }
  }
}
