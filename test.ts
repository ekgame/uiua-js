import { AbstractBackend } from "./lib/backend";
import { runString, UiuaRuntime } from "./lib/main";

class TestBackend extends AbstractBackend {
    constructor(private prefix: string) {
        super();
    }

    printStrStdout(str: string) {
        console.log(this.prefix + ' ' + str);
    }

    printStrStderr(str: string) {
        console.error(this.prefix + ' ' + str);
    }
}

const runtime = new UiuaRuntime();
runtime.setBackend(new TestBackend("[test 1]"));
runtime.setExecutionLimit(5)

const result = runString(runtime, `
    firstrev [1 2 3]
`);

console.log(result);