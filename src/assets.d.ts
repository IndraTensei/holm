// Vite's `?raw` imports: a file's text as a string (harness logos in src/assets/harness).
declare module '*?raw' {
  const text: string;
  export default text;
}

// Holm's version from package.json, put in by Vite (vite.config.ts).
declare const __APP_VERSION__: string;
