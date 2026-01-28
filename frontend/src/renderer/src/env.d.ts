/// <reference types="vite/client" />

declare global {
  interface Window {
    api: {
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      save: (filename: string) => void;
      onUserKeydown: (keyData: string, cursorPos: number | undefined) => void;
      onRemoveRequest: (callback: (position: number) => void) => void;
      onInsertRequest: (
        callback: (position: number, char: string) => void,
      ) => void;
      onFullSync: (callback: (new_text: string) => void) => void;
    };
  }
}

export {};
