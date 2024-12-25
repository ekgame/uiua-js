import {
    prettyFormatValue,
} from "../crate/pkg/uiua_js";

interface UiuaArray<T> extends Array<T | UiuaArray<T>> { }

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

export class UiuaValue {
    private constructor(
        private _data: any,
        private _shape: number[],
        private _label: string | null,
        private _keys: UiuaValue | null,
        private _type: UiuaType,
    ) { }

    get data() {
        return this._data;
    }

    get shape() {
        return this._shape;
    }

    get label() {
        return this._label;
    }

    get keys() {
        return this._keys;
    }

    get type() {
        return this._type;
    }

    box(): UiuaValue {
        return new UiuaValue(this, [], this.label, null, "box");
    }

    unbox(): UiuaValue {
        if (this.type !== "box") {
            return this;
        }

        if (this.shape.length !== 0) {
            throw new Error("Cannot unbox a non-scalar value");
        }

        return this._data;
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
        let data = flattenArray(this.data);

        if (this.type === "box") {
            data = (data as unknown as UiuaValue[])
                .map((value: UiuaValue) => value.toModel());
        } else if (this.type === "char") {
            data = [data];
        }

        return {
            type: this.type,
            shape: this.shape,
            label: this.label,
            keys: this.keys ? this.keys.toModel() : null,
            data: flattenArray(data) as any,
        };
    }

    prettyFormat(): string {
        console.log(this.toModel());
        return prettyFormatValue(this.toModel());
    }

    static fromModel(model: UiuaValueModel): UiuaValue {
        let data: any[] = model.data;

        if (model.type === "box") {
            data = data.map(UiuaValue.fromModel);
        }

        return new UiuaValue(
            reshapeArray(data, model.shape, model.type),
            model.shape,
            model.label || null,
            model.keys ? UiuaValue.fromModel(model.keys) : null,
            model.type,
        );
    }

    static fromNumber(number: number) {
        return new UiuaValue(number, [], null, null, "number");
    }

    static fromNumberArray(array: UiuaArray<number>) {
        const shape = getShape(array);
        if (shape === null) {
            throw new Error("Invalid shape of the array");
        }

        return new UiuaValue(array, shape, null, null, "number");
    }

    static fromBooleanArray(array: UiuaArray<boolean>) {
        const shape = getShape(array);
        if (shape === null) {
            throw new Error("Invalid shape of the array");
        }

        const booleansAsNumbers = flattenArray(array).map((value) => (value ? 1 : 0));
        const modifiedArray = reshapeArray(booleansAsNumbers, shape, "number");

        return new UiuaValue(modifiedArray, shape, null, null, "number");
    }

    static fromString(string: string) {
        return new UiuaValue(string, [string.length], null, null, "char");
    }
}

function reshapeArray(array: any[], shape: number[], type: string): any[] {
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

function flattenArray<T>(arr: UiuaArray<T>): T[] {
    if (typeof arr === "string") {
        return arr;
    }

    if (!Array.isArray(arr)) {
        return [arr];
    }

    return (arr as T[][]).reduce((acc, val) => {
        const elements = flattenArray(val);
        for (const element of elements) {
            acc.push(element);
        }
        return acc;
    }, []);
}