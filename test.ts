import { runString, UiuaRuntime, UiuaValue } from "./lib/main";

const code = `
    Foo = 5
    Bar = + 3
    7
`;

const runtime = new UiuaRuntime();
const result = runString(runtime, code);
console.log(result);

const runtime2 = new UiuaRuntime();
runtime2.setCompiler(result.compiler);
const code2 = `
    Foo Bar 6
    Baz = 44
`;
const result2 = runString(runtime2, code2);
console.log(result2);

const runtime3 = new UiuaRuntime();
runtime3.setCompiler(result2.compiler);
const result3 = runString(runtime3, 'Baz');
console.log(result3);