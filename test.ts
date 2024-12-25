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
const result = runString(runtime, `
    Foo = "Hello world"
    &p "Initialized"
`);

const runtime2 = new UiuaRuntime();
runtime2.setCompiler(result.compiler);
runtime2.setBackend(new TestBackend("[test 3]"));
const results2 = runString(runtime2, `
map {} {}
insert bothbox "this" range10
`);

results2.stack.forEach(value => {
    console.log(value.prettyFormat());
});
