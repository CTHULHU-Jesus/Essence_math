import init, { run_app } from "./pkg/web.js";
async function main() {
  await init("web/pkg/web.wasm");
  run_app();
}
main();
