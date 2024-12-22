import { runString, UiuaRuntime, UiuaValue } from "./lib/main";

const runtime = new UiuaRuntime();

runtime.addBinding("MyAddWithMessage", 2, 1, (uiua) => {
    const val1 = uiua.pop().asNumber();
    const val2 = uiua.pop().asNumber();
    console.log('Hello from JS!');
    const result = UiuaValue.fromNumber(val1 + val2);
    uiua.push(result);
});

let result = runString(`-`, [
    UiuaValue.fromNumber(5),
    UiuaValue.fromNumber(3),
], runtime);

console.log('Result from Uiua: ', result);