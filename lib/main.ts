import init, {
  format_internal,
  FormatConfigStruct,
  test_run,
} from "../crate/pkg/uiua_js";

// @ts-ignore
await init();

interface FormatConfig {
  trailingNewLine: boolean;
  commentSpaceAfterHash: boolean;
  multilineIndent: number;
  alignComments: boolean;
  indentItemImports: boolean;
}

interface DocumentLocation {
  line: number;
  column: number;
}

interface DocumentSpan {
  from: DocumentLocation;
  to: DocumentLocation;
}

interface GlyphMapping {
  spanFrom: DocumentSpan;
  spanTo: DocumentSpan;
}

interface FormatOutput {
  output: string;
  mappings: GlyphMapping[];
}

export function format(
  code: string,
  config?: Partial<FormatConfig>,
): FormatOutput {
  let configStruct = new FormatConfigStruct();

  config = config || {};
  if (config.trailingNewLine !== undefined) {
    configStruct = configStruct.with_trailing_newline(
      config.trailingNewLine,
    );
  }

  if (config.commentSpaceAfterHash !== undefined) {
    configStruct = configStruct.with_comment_space_after_hash(
      config.commentSpaceAfterHash,
    );
  }

  if (config.multilineIndent !== undefined) {
    configStruct = configStruct.with_multiline_indent(
      config.multilineIndent,
    );
  }

  if (config.alignComments !== undefined) {
    configStruct = configStruct.with_align_comments(config.alignComments);
  }

  if (config.indentItemImports !== undefined) {
    configStruct = configStruct.with_indent_item_imports(
      config.indentItemImports,
    );
  }

  const results = format_internal(code, configStruct);

  return {
    output: results.output as string,
    mappings: results.glyph_map as GlyphMapping[],
  };
}

interface UiuaArray<T> extends Array<T | UiuaArray<T>> {}

interface UiuaValueBase {
  type: 'number' | 'char' | 'box' | 'complex'
  shape: number[]
  label: string|null
  keys: UiuaValue|null
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

export function test(code: string) {
    return test_run(code).map(formatResult);
}