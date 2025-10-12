/// <reference types="vite/client" />

export {}

declare global {
  interface Window {
    api: {
      minimize: () => void,
      maximize: () => void,
      close: () => void,
    }
  }
}
