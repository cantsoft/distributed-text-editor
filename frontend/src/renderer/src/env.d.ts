/// <reference types="vite/client" />

import { Doc } from "../../../native";
type DocAPI = Omit<Doc, "constructor">;

declare global {
  interface Window {
    api: {
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      createDocument: () => DocAPI;
    };
  }
}
