import esbuild from 'esbuild';

esbuild.build({
  entryPoints: ['./index.ts'],
  outdir: './dist',
  bundle: true,
  format: 'esm',
  splitting: true,
  chunkNames: '[name]',
  minify: true,
  loader: {
    '.ts': 'ts'
  },
  tsconfig: './tsconfig.json',
  platform: 'node',
  sourcemap: true
}).catch(() => process.exit(1));
