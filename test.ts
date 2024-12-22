import { runString, UiuaRuntime, UiuaValue } from "./lib/main";

const runtime = new UiuaRuntime();

runtime.addBinding("MyAddWithMessage", 2, 1, (uiua) => {
    const val1 = uiua.pop().asNumber();
    const val2 = uiua.pop().asNumber();
    console.log('Hello from JS!');
    const result = UiuaValue.fromNumber(val1 + val2);
    uiua.push(result);
});

let result = runString(`
    map {} {}
    insert∩□ "foo" "bar"
    insert∩□ "baz" ⇡ 10
    insert∩□ "third" ↯ 1_2_3 ⇡ 10    
`, runtime);
console.log('Result from Uiua: ', result);