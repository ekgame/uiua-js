import { format, test, getShape } from "./lib/main";

// const code = 'resh3_4rang12';
// const formatted = format(code, {
//     trailingNewLine: true,
//     commentSpaceAfterHash: true,
//     multilineIndent: 2,
//     alignComments: true,
//     indentItemImports: true
// });

// console.log(test(`9 + 3 5`));
// console.log(test(`range10`));
// console.log(test(`{"123" "asdqwe" [1_2 3_4]}`));
// console.log(test(`↯ 2_inf range10`));
// console.log(test(`↯ 2_2 {"123" "asdqwe" [1_2 3_4]}`));
// console.log(test(`div 2 ↯ 2_inf range10`));
// console.log(test(`$test + @a range10`));

// console.log(getShape(1))
// console.log(getShape([1, 2]))
// console.log(getShape([[1, 2], [3, 4]]))
// console.log(getShape([[[1], [2], [6]], [[3], [4], [5]]]))
// console.log(getShape(["", ""]))
console.log(getShape([[{value: [1, 2, 3]}, {value: "123"}], [{value: [1, 2, 3]}, {value: "123"}]]))

// console.log(flattenArray([[[1], [2], [3]], [[4], [5], [6]]]))

console.log(test(`$asd map {"hello world" "test" "foo"} {"universe" "pog" "bar"}`));
