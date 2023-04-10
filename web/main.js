import init, { run_app } from "./pkg/web.js";
async function main() {
  await init("/pkg/web_bg.wasm");
  run_app();
}
main();
