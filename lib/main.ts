import init, { format_internal, FormatConfigStruct } from '../crate/pkg/uiua_js';

// @ts-ignore
await init();

interface FormatConfig {
    trailingNewLine: boolean,
    commentSpaceAfterHash: boolean,
    multilineIndent: number,
    alignComments: boolean,
    indentItemImports: boolean
}

interface DocumentLocation {
    line: number,
    column: number
}

interface DocumentSpan {
    from: DocumentLocation,
    to: DocumentLocation
}

interface GlyphMapping {
    spanFrom: DocumentSpan,
    spanTo: DocumentSpan,
}

interface FormatOutput {
    output: string,
    mappings: GlyphMapping[],
}

export function format(code: string, config?: Partial<FormatConfig>): FormatOutput {
    let configStruct = new FormatConfigStruct();

    config = config || {};
    if (config.trailingNewLine !== undefined) {
        configStruct = configStruct.with_trailing_newline(config.trailingNewLine);
    }

    if (config.commentSpaceAfterHash !== undefined) {
        configStruct = configStruct.with_comment_space_after_hash(config.commentSpaceAfterHash);
    }

    if (config.multilineIndent !== undefined) {
        configStruct = configStruct.with_multiline_indent(config.multilineIndent);
    }

    if (config.alignComments !== undefined) {
        configStruct = configStruct.with_align_comments(config.alignComments);
    }

    if (config.indentItemImports !== undefined) {
        configStruct = configStruct.with_indent_item_imports(config.indentItemImports);
    }

    const results = format_internal(code, configStruct);

    return {
        output: results.output as string,
        mappings: results.glyph_map as GlyphMapping[],
    };
}