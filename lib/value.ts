import {
    NativeValueWrapper,
} from "../crate/pkg/uiua_js";

interface SmartValueBase {
    type: "png" | "gif" | "wav" | "normal"
}

interface SmartValuePng extends SmartValueBase {
    type: "png"
    data: Uint8Array
}

interface SmartValueGif extends SmartValueBase {
    type: "gif"
    data: Uint8Array
}

interface SmartValueWav extends SmartValueBase {
    type: "wav"
    data: Uint8Array
}

interface SmartValueNormal extends SmartValueBase {
    type: "normal"
    data: NativeValueWrapper
}

export type SmartValue = SmartValuePng | SmartValueGif | SmartValueWav | SmartValueNormal

type UiuaType = "char" | "number" | "complex" | "box";

export class UiuaValue {
    private constructor(
        private internal: NativeValueWrapper,
    ) {}

    get data(): any {
        let data = this.internal.data();
        return reshapeArray(data, this.shape, this.type);
    }

    get shape(): number[] {
        return Array.from(this.internal.shape());
    }

    get type(): UiuaType {
        return this.internal.type() as any;
    }

    get internalWrapper(): NativeValueWrapper {
        return this.internal;
    }

    show(): string {
        return this.internal.show();
    }

    smartValue(): SmartValue {
        return this.internal.smartValue();
    }

    static fromWrapper(internal: NativeValueWrapper): UiuaValue {
        return new UiuaValue(internal);
    }
}

function reshapeArray(array: any, shape: number[], type: string): any {
    let index = 0;

    function nest(currentShape: number[]) {
        if (currentShape.length === 0) {
            return array[index++];
        }

        const size = currentShape[0];
        const restShape = currentShape.slice(1);
        if (type == 'char' && restShape.length === 0) {
            array as unknown as string;
            const result = array.slice(index, index + size);
            index += size;
            return result;
        }

        const result: any[] = [];

        for (let i = 0; i < size; i++) {
            result.push(nest(restShape));
        }

        return result;
    }

    return nest(shape);
}

function getShape(item: any): number[] | null {
    if (typeof item === "number" || typeof item === "boolean") {
        // Scalars have no dimensions
        return [];
    }

    if (typeof item === "object" && item !== null && "value" in item) {
        // Boxed values have no dimensions
        return [];
    }

    if (typeof item === "string") {
        // Strings have a single dimension, the length of the string
        return [item.length];
    }

    if (!Array.isArray(item)) {
        // Invalid type
        return null;
    }

    if (item.length === 0) {
        return [0];
    }

    const innerValueShapes = item.map(getShape);
    if (innerValueShapes.some((shape) => shape === null)) {
        // Invalid shape of some inner value
        return null;
    }

    const firstShape = innerValueShapes[0] as number[];
    const firstShapeString = JSON.stringify(innerValueShapes[0]);
    if (innerValueShapes.some((shape) => JSON.stringify(shape) !== firstShapeString)) {
        // Inner values have different shapes
        return null;
    }

    if (firstShapeString === "[]") {
        return [item.length];
    }

    return [item.length, ...firstShape];
}

function flattenArray<T>(arr: any, type: UiuaType): any {
    if (typeof arr === "string") {
        return arr;
    }

    if (!Array.isArray(arr)) {
        return [arr];
    }

    if (type === 'char') {
        let accumulator = '';
        for (const element of arr) {
            const flattened = flattenArray(element, type);
            for (const value of flattened) {
                accumulator += value;
            }
        }
        return accumulator;
    } else {
        return (arr as T[][]).reduce((acc, val) => {
            const elements = flattenArray(val, type);
            for (const element of elements) {
                acc.push(element);
            }
            return acc;
        }, []);
    }
}