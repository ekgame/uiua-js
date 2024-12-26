import {
    prettyFormatValue,
    toSmartValue,
} from "../crate/pkg/uiua_js";

type UiuaArray<T> = T
    | T[]
    | T[][]
    | T[][][]
    | T[][][][]
    | T[][][][][]
    | T[][][][][][]
    | T[][][][][][][]
    | T[][][][][][][][]
    | T[][][][][][][][][]
    | T[][][][][][][][][][]
    | T[][][][][][][][][][][]
    | T[][][][][][][][][][][][]
    | T[][][][][][][][][][][][][]
    | T[][][][][][][][][][][][][][]
    | T[][][][][][][][][][][][][][][]
    | T[][][][][][][][][][][][][][][][]
    | T[][][][][][][][][][][][][][][][][]
    | T[][][][][][][][][][][][][][][][][][]
    | T[][][][][][][][][][][][][][][][][][][];

type UiuaType = 'number' | 'char' | 'box' | 'complex'

interface UiuaValueBase {
    type: UiuaType
    shape: number[]
    label: string | null
    keys: UiuaValueModel | null
}

interface UiuaValueNumber extends UiuaValueBase {
    type: 'number'
    data: UiuaArray<number>
}

interface UiuaValueChar extends UiuaValueBase {
    type: 'char'
    data: UiuaArray<string>
}

interface UiuaValueComplex extends UiuaValueBase {
    type: 'complex'
    data: UiuaArray<[number, number]>
}

interface UiuaValueBox extends UiuaValueBase {
    type: 'box'
    data: UiuaArray<UiuaValueModel>
}

export type UiuaValueModel = UiuaValueNumber | UiuaValueChar | UiuaValueComplex | UiuaValueBox

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
    data: UiuaValueModel
}

export type SmartValue = SmartValuePng | SmartValueGif | SmartValueWav | SmartValueNormal

export class UiuaValue {
    private constructor(
        private _model: UiuaValueModel,
    ) { }

    get data() {
        return reshapeArray(this._model.data, this._model.shape, this._model.type);
    }

    get shape() {
        return this._model.shape;
    }

    get label() {
        return this._model.label;
    }

    get keys() {
        return this._model.keys ? UiuaValue.fromModel(this._model.keys) : null;
    }

    get type() {
        return this._model.type;
    }

    box(): UiuaValue {
        return new UiuaValue({
            type: "box",
            data: [this],
            shape: [],
            label: null,
            keys: null,
        });
    }

    unbox(): UiuaValue {
        if (this.type !== "box") {
            return this;
        }

        if (this.shape.length !== 0) {
            throw new Error("Cannot unbox a non-scalar value");
        }

        return this.data[0];
    }

    asNumber(): number {
        if (this.type !== "number") {
            throw new Error("Can not convert a nun-number value to a number");
        }

        if (this.shape.length !== 0) {
            throw new Error("Can not convert a non-scalar value to a number");
        }

        return this.data;
    }

    asNumberArray(): UiuaArray<number> {
        if (this.type !== "number") {
            throw new Error("Can not convert a non-number value to a number array");
        }

        return this.data;
    }

    asString(): string {
        if (this.type !== "char") {
            throw new Error("Can not convert a non-char value to a string");
        }

        if (this.shape.length !== 1) {
            throw new Error("Can not convert a non-1D char array to a string");
        }

        return this.data;
    }

    toModel(): UiuaValueModel {
        return this._model;
    }

    prettyFormat(): string {
        return prettyFormatValue(this.toModel());
    }

    toSmartValue(): SmartValue {
        const result = toSmartValue(this.toModel());

        if (result.type === "normal") {
            return {
                type: "normal",
                data: UiuaValue.fromModel(result.data),
            };
        }

        return result;
    }

    static fromModel(model: UiuaValueModel): UiuaValue {
        return new UiuaValue(model);
    }

    static fromNumber(number: number) {
        return new UiuaValue({
            type: "number",
            data: [number],
            shape: [],
            label: null,
            keys: null,
        });
    }

    static fromNumberArray(array: UiuaArray<number>) {
        const shape = getShape(array);
        if (shape === null) {
            throw new Error("Invalid shape of the array");
        }

        return new UiuaValue({
            type: "number",
            data: array,
            shape,
            label: null,
            keys: null,
        });
    }

    static fromBooleanArray(array: UiuaArray<boolean>) {
        const shape = getShape(array);
        if (shape === null) {
            throw new Error("Invalid shape of the array");
        }

        return new UiuaValue({
            type: "number",
            data: flattenArray(array, "number").map((value) => (value ? 1 : 0)),
            shape,
            label: null,
            keys: null,
        });
    }

    static fromString(string: string) {
        return new UiuaValue({
            type: "char",
            data: string,
            shape: [string.length],
            label: null,
            keys: null,
        });
    }

    static fromStringArray(array: UiuaArray<string>) {
        const shape = getShape(array);
        if (shape === null) {
            throw new Error("Invalid shape of the array");
        }

        return new UiuaValue({
            type: "char",
            data: flattenArray(array, "char"),
            shape,
            label: null,
            keys: null,
        });
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