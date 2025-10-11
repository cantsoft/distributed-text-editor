// import { defineConfig } from 'vite';

// // https://vitejs.dev/config
// export default defineConfig({});



import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  assetsInclude: ['**/*.node'],

  build: {
    rollupOptions: {
      external: [
        'electron',
        'path',
        'fs',
        path.resolve(__dirname, '../native/index.node'),
      ],
    },
  },

  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV ?? 'development'),
  },
});
