import { AbstractBackend } from "./lib/backend";
import { runString, UiuaRuntime, UiuaValue } from "./lib/main";

class TestBackend extends AbstractBackend {
    printStrStdout(str: string) {
        console.log(str);
    }

    printStrStderr(str: string) {
        console.error(str);
    }
}

const runtime = new UiuaRuntime();
runtime.setBackend(new TestBackend());

const result = runString(runtime, `
    &w "Hello from Uiua" 2
    # &w "Hello from Uiua" 1
`);

console.log(result);
