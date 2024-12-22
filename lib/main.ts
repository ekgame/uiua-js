import init, {
  run,
} from "../crate/pkg/uiua_js";

import { UiuaRuntime } from "./runtime";
import { UiuaValue } from "./value";

export { UiuaRuntime } from "./runtime";
export { UiuaValue } from "./value";

// @ts-ignore
await init();

export function test(code: string, runtime: UiuaRuntime): UiuaValue[] {
  return run(code, runtime.internal).map(UiuaValue.fromModel);
}