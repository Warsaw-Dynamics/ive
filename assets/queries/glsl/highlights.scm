; IVE highlight query for GLSL. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @GLSL.PARENTHESES
["[" "]"] @GLSL.BRACKETS
["{" "}"] @GLSL.BRACES
"," @GLSL.COMMA
"." @GLSL.DOT
";" @GLSL.SEMICOLON
[
  "=" "+" "-" "*" "/" "%" "==" "!=" "<" ">" "<=" ">=" "&&" "||" "!" "&" "|" "^" "~" "<<" ">>" "?" ":"
  "++" "--" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>="
] @GLSL.OPERATION_SIGN

(number_literal) @GLSL.NUMBER
((comment) @GLSL.LINE_COMMENT
 (#match? @GLSL.LINE_COMMENT "^//"))
((comment) @GLSL.BLOCK_COMMENT
 (#match? @GLSL.BLOCK_COMMENT "^/\\*"))

[
  "break" "case" "const" "continue" "default" "do" "else" "for" "if" "return" "struct" "subroutine" "switch" "while"
] @GLSL.KEYWORD
[(true) (false)] @GLSL.KEYWORD
((identifier) @GLSL.KEYWORD
 (#eq? @GLSL.KEYWORD "discard"))
[
  "attribute" "buffer" "centroid" "coherent" "flat" "highp" "in" "inout" "invariant" "layout" "lowp" "mediump"
  "noperspective" "out" "patch" "precise" "precision" "readonly" "restrict" "sample" "shared" "smooth" "uniform"
  "varying" "volatile" "writeonly" (extension_storage_class)
] @GLSL.QUALIFIER
(primitive_type) @GLSL.PRIMITIVE_TYPE

["#define" "#elif" "#else" "#endif" "#if" "#ifdef" "#ifndef" "#include" "defined"] @GLSL.PREPROCESSOR_DIRECTIVE
(preproc_directive) @GLSL.PREPROCESSOR_DIRECTIVE

(type_identifier) @GLSL.TYPE
((type_identifier) @GLSL.PRIMITIVE_TYPE
 (#match? @GLSL.PRIMITIVE_TYPE
   "^([biud]?vec[234]|d?mat[234](x[234])?|[iu]?(sampler|image|texture)(1D|2D|3D|Cube|2DRect|Buffer|2DMS)(Array)?(Shadow)?|sampler|samplerShadow|atomic_uint|subpassInput(MS)?)$"))
(declaration (identifier) @GLSL.TYPE (field_declaration_list))
((identifier) @GLSL.CONSTANT
 (#match? @GLSL.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(field_identifier) @GLSL.FIELD
(qualifier (identifier) @GLSL.LAYOUT_QUALIFIER)

; a declaration is global unless it sits in a block or a for loop's initializer; no pattern is rooted at the
; file node, which would make the query of any line walk every top-level node of the file
(declaration
  declarator: [
    (identifier) @GLSL.GLOBAL_VARIABLE
    (init_declarator declarator: (identifier) @GLSL.GLOBAL_VARIABLE)
    (init_declarator declarator: (array_declarator declarator: (identifier) @GLSL.GLOBAL_VARIABLE))
    (array_declarator declarator: (identifier) @GLSL.GLOBAL_VARIABLE)
  ])
(compound_statement
  (declaration
    declarator: [
      (identifier) @GLSL.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @GLSL.LOCAL_VARIABLE)
      (init_declarator declarator: (array_declarator declarator: (identifier) @GLSL.LOCAL_VARIABLE))
      (array_declarator declarator: (identifier) @GLSL.LOCAL_VARIABLE)
    ]))
(for_statement
  initializer: (declaration
    declarator: [
      (identifier) @GLSL.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @GLSL.LOCAL_VARIABLE)
      (init_declarator declarator: (array_declarator declarator: (identifier) @GLSL.LOCAL_VARIABLE))
      (array_declarator declarator: (identifier) @GLSL.LOCAL_VARIABLE)
    ]))
(parameter_declaration
  declarator: [
    (identifier) @GLSL.PARAMETER
    (array_declarator declarator: (identifier) @GLSL.PARAMETER)
  ])

(function_declarator declarator: (identifier) @GLSL.FUNCTION_DECLARATION)
(call_expression function: (identifier) @GLSL.FUNCTION_CALL)
((call_expression function: (identifier) @GLSL.TYPE)
 (#match? @GLSL.TYPE "^[A-Z]"))
((call_expression function: (identifier) @GLSL.PRIMITIVE_TYPE)
 (#match? @GLSL.PRIMITIVE_TYPE "^([biud]?vec[234]|d?mat[234](x[234])?|float|double|int|uint|bool)$"))
((call_expression function: (identifier) @GLSL.BUILTIN_FUNCTION)
 (#any-of? @GLSL.BUILTIN_FUNCTION
   "abs" "acos" "acosh" "all" "any" "asin" "asinh" "atan" "atanh" "atomicAdd" "atomicAnd" "atomicCompSwap"
   "atomicCounter" "atomicCounterDecrement" "atomicCounterIncrement" "atomicExchange" "atomicMax" "atomicMin"
   "atomicOr" "atomicXor" "barrier" "bitCount" "bitfieldExtract" "bitfieldInsert" "bitfieldReverse" "ceil" "clamp"
   "cos" "cosh" "cross" "dFdx" "dFdxCoarse" "dFdxFine" "dFdy" "dFdyCoarse" "dFdyFine" "degrees" "determinant"
   "distance" "dot" "EmitStreamVertex" "EmitVertex" "EndPrimitive" "EndStreamPrimitive" "equal" "exp" "exp2"
   "faceforward" "findLSB" "findMSB" "floatBitsToInt" "floatBitsToUint" "floor" "fma" "fract" "frexp" "fwidth"
   "fwidthCoarse" "fwidthFine" "greaterThan" "greaterThanEqual" "groupMemoryBarrier" "imageAtomicAdd" "imageLoad"
   "imageSize" "imageStore" "imulExtended" "intBitsToFloat" "interpolateAtCentroid" "interpolateAtOffset"
   "interpolateAtSample" "inverse" "inversesqrt" "isinf" "isnan" "ldexp" "length" "lessThan" "lessThanEqual" "log"
   "log2" "matrixCompMult" "max" "memoryBarrier" "memoryBarrierAtomicCounter" "memoryBarrierBuffer"
   "memoryBarrierImage" "memoryBarrierShared" "min" "mix" "mod" "modf" "normalize" "not" "notEqual" "outerProduct"
   "packDouble2x32" "packHalf2x16" "packSnorm2x16" "packSnorm4x8" "packUnorm2x16" "packUnorm4x8" "pow" "radians"
   "reflect" "refract" "round" "roundEven" "sign" "sin" "sinh" "smoothstep" "sqrt" "step" "subpassLoad" "tan"
   "tanh" "texelFetch" "texelFetchOffset" "texture" "texture2D" "texture2DLod" "texture2DProj" "textureCube"
   "textureGather" "textureGatherOffset" "textureGatherOffsets" "textureGrad" "textureGradOffset" "textureLod"
   "textureLodOffset" "textureOffset" "textureProj" "textureProjGrad" "textureProjGradOffset" "textureProjLod"
   "textureProjLodOffset" "textureProjOffset" "textureQueryLevels" "textureQueryLod" "textureSamples" "textureSize"
   "transpose" "trunc" "uaddCarry" "uintBitsToFloat" "umulExtended" "unpackDouble2x32" "unpackHalf2x16"
   "unpackSnorm2x16" "unpackSnorm4x8" "unpackUnorm2x16" "unpackUnorm4x8" "usubBorrow"))
((identifier) @GLSL.BUILTIN_VARIABLE
 (#match? @GLSL.BUILTIN_VARIABLE "^gl_"))

(preproc_def name: (identifier) @GLSL.MACRO)
(preproc_function_def name: (identifier) @GLSL.MACRO)
(preproc_ifdef name: (identifier) @GLSL.MACRO)
(preproc_params (identifier) @GLSL.PARAMETER)
(preproc_extension extension: (identifier) @GLSL.MACRO)
(extension_behavior) @GLSL.KEYWORD
