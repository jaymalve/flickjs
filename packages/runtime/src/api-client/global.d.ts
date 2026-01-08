// Minimal type declaration for process.env (browser-safe)
declare namespace NodeJS {
  interface ProcessEnv {
    [key: string]: string | undefined;
  }
}

declare const process: {
  env: NodeJS.ProcessEnv;
} | undefined;

// Type declaration for Vite's import.meta.env
interface ImportMetaEnv {
  [key: string]: string | undefined;
  VITE_FLICK_API_URL?: string;
}

interface ImportMeta {
  env?: ImportMetaEnv;
}
