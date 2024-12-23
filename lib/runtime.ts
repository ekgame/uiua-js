import {
    CompilerRef,
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
     * 
     * Note: that this is ignored if a custom compiler is set.
     */
    addBinding(name: string, inputs: number, outputs: number, callback: (uiua: Uiua) => void) {
        this.internal.addBinding(name, inputs, outputs, (ref: UiuaRef) => {
            callback(new Uiua(ref));
        });
    }

    /**
     * Set a custom compiler to the runtime.
     * This is useful for running Uiua code with the context of some previous code.
     * 
     * Note: that this discards any custom bindings defined for this runtime.
     */
    setCompiler(compiler: CompilerRef) {
        this.internal.setCompiler(compiler);
    }
}