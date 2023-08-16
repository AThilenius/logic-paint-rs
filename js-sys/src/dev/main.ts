/// Entry point for the dev site, used to develop/test the library.
import init, { say_hello } from "core";
import { doTheThings } from "~/lib";

async function main() {
  await init();
  say_hello();
}

main();

console.log("Hello dev code");
void doTheThings();
