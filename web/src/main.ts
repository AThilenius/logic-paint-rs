/// Entry point for the dev site, used to develop/test the library.
import init, { web_main } from "core";

async function main() {
  await init();
  web_main();
}

main();
