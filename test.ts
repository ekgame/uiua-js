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
console.log("Running code:");
const result = runtime.runString(`
    Music
    Lena
`);
console.log(result);
console.log(result.stack.map(value => value.smartValue()));

// const test = UiuaValue.fromStringArray(["foo", "bar", "baz", "qux"]);
// console.log(test.toSmartValue());