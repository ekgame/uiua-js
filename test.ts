import { format } from "./lib/main";

const code = 'resh3_4rang12';
const formatted = format(code, {
    trailingNewLine: true,
    commentSpaceAfterHash: true,
    multilineIndent: 2,
    alignComments: true,
    indentItemImports: true
});
console.log(formatted);