import { AbstractBackend } from "./lib/backend";
import { UiuaRuntime, UiuaValue } from "./lib/main";

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
const result = runtime.runString(`
    Music
    Lena
    box 5
`);
console.log(result.stack.map(x => x.toSmartValue()));

// const test = UiuaValue.fromStringArray(["foo", "bar", "baz", "qux"]);
// console.log(test.toSmartValue());