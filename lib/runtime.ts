import {
    CompilerRef,
    UiuaRef,
    UiuaRuntimeInternal,
} from "../crate/pkg/uiua_js";
import { AbstractBackend } from "./backend";

import { UiuaValue } from "./value";

/**
 * An instance of this class is available for the callbacks of custom bindings
 * to interact with Uiua stack at runtime.
 */
class Uiua {
    private ref: UiuaRef;

    constructor(ref: UiuaRef) {
        this.ref = ref;
    }

    pop() {
        return UiuaValue.fromModel(this.ref.pop());
    }

    push(value: UiuaValue) {
        this.ref.push(value.toModel());
    }
}

/**
 * The context for running Uiua code.
 */
export class UiuaRuntime {
    internal: UiuaRuntimeInternal;

    constructor() {
        this.internal = new UiuaRuntimeInternal();
    }

    /**
     * Add a custom binding to the runtime. Allows calling JavaScript code from Uiua runtime.
     * 
     * @param name The name of the binding.
     * @param inputs The number of inputs the binding takes.
     * @param outputs The number of outputs the binding produces.
     * @param callback The callback to run when the binding is called.
     */
    addBinding(name: string, inputs: number, outputs: number, callback: (uiua: Uiua) => void) {
        this.internal.addBinding(name, inputs, outputs, (ref: UiuaRef) => {
            callback(new Uiua(ref));
        });
    }

    /**
     * Set a custom compiler to the runtime. This is useful for running Uiua code with the context of some previous code.
     * 
     * @param compiler The compiler to use.
     */
    setCompiler(compiler: CompilerRef) {
        this.internal.setCompiler(compiler);
    }

    /**
     * Set a custom backend to use for execution.
     * 
     * @param backend The backend to use.
     */
    setBackend(backend: AbstractBackend) {
        let internalBackend = this.internal.getBackend();
        internalBackend = internalBackend.with_print_str_stdout_handler(backend.printStrStdout.bind(backend));
        internalBackend = internalBackend.with_print_str_stderr_handler(backend.printStrStderr.bind(backend));
        this.internal.setBackend(internalBackend);
    }

    setExecutionLimit(seconds: number) {
        this.internal.setExecutionLimitSeconds(seconds);
    }
}