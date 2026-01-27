/// <reference types="vite/client" />

declare global {
  interface Window {
    api: {
      minimize: () => void,
      maximize: () => void,
      close: () => void,
      onUserKeydown(key: string, cursor_pos: number | undefined): unknown,
      save: (filename: string) => void,
    };
  }
}
