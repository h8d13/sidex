import { defineConfig } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';
import * as path from 'path';

export default defineConfig({
  clearScreen: false,
  assetsInclude: ['**/*.wasm'],
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
    fs: {
      allow: ['.', 'extensions', 'node_modules'],
    },
  },
  plugins: [
    viteStaticCopy({
      targets: [
        {
          src: 'extensions',
          dest: '.',
        },
        {
          src: 'extensions-meta.json',
          dest: '.',
        },
        {
          src: 'node_modules/vscode-oniguruma/release/onig.wasm',
          dest: '.',
        },
      ],
    }),
  ],
  envPrefix: ['VITE_', 'TAURI_'],
  resolve: {
    alias: {
      'vs': path.resolve(__dirname, 'src/vs'),
    },
  },
  build: {
    target: ['es2022', 'chrome100', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    chunkSizeWarningLimit: 3000,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('monaco-editor')) {
            return 'monaco-editor';
          }
          if (id.includes('/contrib/notebook/') || id.includes('/services/notebook/')) {
            return 'notebook';
          }
          if (id.includes('/contrib/chat/') || id.includes('/contrib/inlineChat/') ||
              id.includes('/services/chat/') || id.includes('/contrib/mcp/')) {
            return 'chat';
          }
          if (id.includes('/contrib/testing/')) {
            return 'testing';
          }
          if (id.includes('/contrib/debug/')) {
            return 'debug';
          }
          if (id.includes('/contrib/remote/') || id.includes('/services/remote/') ||
              id.includes('/contrib/remoteCodingAgents/')) {
            return 'remote';
          }
          if (id.includes('/contrib/interactive/') || id.includes('/contrib/replNotebook/')) {
            return 'interactive';
          }
          if (id.includes('/contrib/terminal/') || id.includes('/services/terminal/') ||
              id.includes('xterm')) {
            return 'terminal';
          }
          if (id.includes('/contrib/search/') || id.includes('/services/search/') ||
              id.includes('/contrib/searchEditor/')) {
            return 'search';
          }
          if (id.includes('/contrib/scm/') || id.includes('/contrib/git/')) {
            return 'scm';
          }
          if (id.includes('/contrib/extensions/') || id.includes('/services/extensionManagement/')) {
            return 'extensions-mgmt';
          }
          if (id.includes('/contrib/welcomeGettingStarted/') || id.includes('/contrib/welcomeWalkthrough/') ||
              id.includes('/contrib/welcomeViews/') || id.includes('/contrib/welcomeBanner/') ||
              id.includes('/contrib/welcomeAgentSessions/')) {
            return 'welcome';
          }
          if (id.includes('/contrib/mergeEditor/') || id.includes('/contrib/multiDiffEditor/')) {
            return 'diff-editors';
          }
          return undefined;
        },
      },
    },
  },
  optimizeDeps: {
    include: ['vscode-textmate', 'vscode-oniguruma'],
    exclude: ['@tauri-apps/api', '@tauri-apps/plugin-dialog', '@tauri-apps/plugin-fs',
              '@tauri-apps/plugin-clipboard-manager', '@tauri-apps/plugin-shell',
              '@tauri-apps/plugin-notification', '@tauri-apps/plugin-opener'],
  },
  worker: {
    format: 'es',
  },
});
