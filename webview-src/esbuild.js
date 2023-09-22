import esbuild from 'esbuild';

esbuild.build({
  entryPoints: ['./webview-src/index.ts'],
  outdir: './webview-dist',
  bundle: true,
  format: 'esm',
  splitting: true,
  chunkNames: '[name]',
  minify: true,
  loader: {
    '.ts': 'ts'
  },
  tsconfig: './webview-src/tsconfig.json',
  platform: 'node',
  sourcemap: true
}).catch(() => process.exit(1));
