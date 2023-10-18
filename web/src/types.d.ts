declare module "*.wasm?init" {
  const value: () => Promise<WebAssembly.Instance>;
  export = value;
}
