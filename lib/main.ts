import init, {
  CompilerRef,
  runCode,
} from "../crate/pkg/uiua_js";

import { UiuaRuntime } from "./runtime";
import { UiuaValue } from "./value";

export { UiuaRuntime } from "./runtime";
export { UiuaValue } from "./value";

// @ts-ignore
await init();

/**
 * The result after Uiua code is executed.
 */
interface UiuaExecutionResult {
  stack: UiuaValue[];
  compiler: CompilerRef;
  stdout: Uint8Array;
  stderr: Uint8Array;
}

/**
 * Run Uiua code with the given runtime and initial values.
 * 
 * @param runtime The runtime to run the code with.
 * @param code The Uiua code to run.
 * @param initialValues The initial values to start the stack with.
 */
export function runString(
  runtime: UiuaRuntime,
  code: string,
  initialValues: UiuaValue[] = [],
): UiuaExecutionResult {
  const result = runCode(
    code,
    initialValues.map(value => value.toModel()),
    runtime.internal
  );
  
  return {
    stack: result.stack.map(UiuaValue.fromModel),
    compiler: result.compiler,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}
