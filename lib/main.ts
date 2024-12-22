import init, {
  runCode,
} from "../crate/pkg/uiua_js";

import { UiuaRuntime } from "./runtime";
import { UiuaValue } from "./value";

export { UiuaRuntime } from "./runtime";
export { UiuaValue } from "./value";

// @ts-ignore
await init();

export function runString(code: string, initialValues: UiuaValue[], runtime: UiuaRuntime): UiuaValue[] {
  return runCode(
    code,
    initialValues.map(value => value.toModel()),
    runtime.internal
  ).map(UiuaValue.fromModel);
}