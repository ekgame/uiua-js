import { runString, UiuaRuntime, UiuaValue } from "./lib/main";


const runtime = new UiuaRuntime();
runtime.setPrintStrStdoutHandler((str: string) => {
    console.log(str);
});

const result = runString(runtime, `
    &p "Hello from Uiua"
`);
