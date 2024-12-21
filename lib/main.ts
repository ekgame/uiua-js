import init, {
  format_internal,
  FormatConfigStruct,
  JsBinding,
  JsRuntime,
  run,
} from "../crate/pkg/uiua_js";

// @ts-ignore
await init();

interface UiuaArray<T> extends Array<T | UiuaArray<T>> { }

interface UiuaValueBase {
  type: 'number' | 'char' | 'box' | 'complex'
  shape: number[]
  label: string | null
  keys: UiuaValue | null
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
  data: UiuaArray<UiuaValue>
}

type UiuaValue = UiuaValueNumber | UiuaValueChar | UiuaValueComplex | UiuaValueBox

export function getShape(item: any): number[] | null {
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

function formatResult(result: any): UiuaValue {
  let data = result.data;

  if (result.type === "box") {
    data = data.map(formatResult);
  }

  return {
    data: reshapeArray(data, result.shape, result.type),
    shape: result.shape,
    label: result.label || null,
    keys: result.keys ? formatResult(result.keys) : null,
    type: result.type,
  };
}

export interface UiuaRuntime {
  bindings: UiuaJavaScriptBinding[],
}

export interface UiuaJavaScriptBinding {
  name: string,
  signature: [inputs: number, outputs: number],
  callback: (uiua: Uiua) => void,
}

interface Uiua {

}

export function test(code: string, runtime: UiuaRuntime): UiuaValue[] {
  const internalRuntime = new JsRuntime();
  runtime.bindings.forEach(({name, signature, callback}) => {
    const binding = new JsBinding(name, signature[0], signature[1], callback);
    internalRuntime.add_binding(binding);
  });

  return run(code, internalRuntime).map(formatResult);
}