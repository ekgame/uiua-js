import init, {
  runCode,
} from "../crate/pkg/uiua_js";

import { UiuaRuntime } from "./runtime";
import { UiuaValue } from "./value";

export { UiuaRuntime } from "./runtime";
export { UiuaValue } from "./value";

// @ts-ignore
await init();

interface UiuaExecutionResult {
  stack: UiuaValue[];
}

export function runString(code: string, initialValues: UiuaValue[], runtime: UiuaRuntime): UiuaExecutionResult {
  const result = runCode(
    code,
    initialValues.map(value => value.toModel()),
    runtime.internal
  );
  
  return {
    stack: result.stack.map(value => UiuaValue.fromModel(value)),
  }
}