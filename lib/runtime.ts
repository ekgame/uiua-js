import {
    CompilerRef,
    ExternalBackendHandlers,
    UiuaRef,
    UiuaRuntimeInternal,
} from "../crate/pkg/uiua_js";

import { UiuaValue } from "./value";

/**
 * An instance of this class is available for the callbacks of custom bindings
 * to interact with the Uiua at runtime.
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
     * Add a custom binding to the runtime.
     * Allows calling JavaScript code from Uiua runtime.
     */
    addBinding(name: string, inputs: number, outputs: number, callback: (uiua: Uiua) => void) {
        this.internal.addBinding(name, inputs, outputs, (ref: UiuaRef) => {
            callback(new Uiua(ref));
        });
    }

    /**
     * Set a custom compiler to the runtime.
     * This is useful for running Uiua code with the context of some previous code.
     */
    setCompiler(compiler: CompilerRef) {
        this.internal.setCompiler(compiler);
    }

    private modifyBackend(modifier: (backend: ExternalBackendHandlers) => ExternalBackendHandlers) {
        const backend = modifier(this.internal.getBackend());
        this.internal.setBackend(backend);
    }

    setPrintStrStdoutHandler(handler: (str: string) => void) {
        this.modifyBackend(backend => backend.with_print_str_stdout_handler(handler));
    }
    
    setPrintStrStderrHandler(handler: (str: string) => void) {
        this.modifyBackend(backend => backend.with_print_str_stderr_handler(handler));
    }
}