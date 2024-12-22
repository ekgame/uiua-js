import {
    CompilerRef,
    UiuaRef,
    UiuaRuntimeInternal,
} from "../crate/pkg/uiua_js";

import { UiuaValue } from "./value";

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

export class UiuaRuntime {
    internal: UiuaRuntimeInternal;

    constructor() {
        this.internal = new UiuaRuntimeInternal();
    }

    addBinding(name: string, inputs: number, outputs: number, callback: (uiua: Uiua) => void) {
        this.internal.addBinding(name, inputs, outputs, (ref: UiuaRef) => {
            callback(new Uiua(ref));
        });
    }

    setCompiler(compiler: CompilerRef) {
        this.internal.setCompiler(compiler);
    }
}