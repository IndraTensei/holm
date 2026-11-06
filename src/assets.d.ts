// Vite's `?raw` imports: a file's text as a string (harness logos in src/assets/harness).
declare module '*?raw' {
  const text: string;
  export default text;
}
