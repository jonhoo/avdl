#![allow(nonstandard_style)]
// Generated from avro/share/idl_grammar/org/apache/avro/idl/Idl.g4 by ANTLR 4.13.2
use antlr4rust::tree::ParseTreeListener;
use super::idlparser::*;

pub trait IdlListener<'input> : ParseTreeListener<'input,IdlParserContextType>{
/**
 * Enter a parse tree produced by {@link IdlParser#idlFile}.
 * @param ctx the parse tree
 */
fn enter_idlFile(&mut self, _ctx: &IdlFileContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#idlFile}.
 * @param ctx the parse tree
 */
fn exit_idlFile(&mut self, _ctx: &IdlFileContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#protocolDeclaration}.
 * @param ctx the parse tree
 */
fn enter_protocolDeclaration(&mut self, _ctx: &ProtocolDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#protocolDeclaration}.
 * @param ctx the parse tree
 */
fn exit_protocolDeclaration(&mut self, _ctx: &ProtocolDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#protocolDeclarationBody}.
 * @param ctx the parse tree
 */
fn enter_protocolDeclarationBody(&mut self, _ctx: &ProtocolDeclarationBodyContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#protocolDeclarationBody}.
 * @param ctx the parse tree
 */
fn exit_protocolDeclarationBody(&mut self, _ctx: &ProtocolDeclarationBodyContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#namespaceDeclaration}.
 * @param ctx the parse tree
 */
fn enter_namespaceDeclaration(&mut self, _ctx: &NamespaceDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#namespaceDeclaration}.
 * @param ctx the parse tree
 */
fn exit_namespaceDeclaration(&mut self, _ctx: &NamespaceDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#mainSchemaDeclaration}.
 * @param ctx the parse tree
 */
fn enter_mainSchemaDeclaration(&mut self, _ctx: &MainSchemaDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#mainSchemaDeclaration}.
 * @param ctx the parse tree
 */
fn exit_mainSchemaDeclaration(&mut self, _ctx: &MainSchemaDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#identifier}.
 * @param ctx the parse tree
 */
fn enter_identifier(&mut self, _ctx: &IdentifierContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#identifier}.
 * @param ctx the parse tree
 */
fn exit_identifier(&mut self, _ctx: &IdentifierContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#schemaProperty}.
 * @param ctx the parse tree
 */
fn enter_schemaProperty(&mut self, _ctx: &SchemaPropertyContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#schemaProperty}.
 * @param ctx the parse tree
 */
fn exit_schemaProperty(&mut self, _ctx: &SchemaPropertyContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#importStatement}.
 * @param ctx the parse tree
 */
fn enter_importStatement(&mut self, _ctx: &ImportStatementContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#importStatement}.
 * @param ctx the parse tree
 */
fn exit_importStatement(&mut self, _ctx: &ImportStatementContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#namedSchemaDeclaration}.
 * @param ctx the parse tree
 */
fn enter_namedSchemaDeclaration(&mut self, _ctx: &NamedSchemaDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#namedSchemaDeclaration}.
 * @param ctx the parse tree
 */
fn exit_namedSchemaDeclaration(&mut self, _ctx: &NamedSchemaDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#fixedDeclaration}.
 * @param ctx the parse tree
 */
fn enter_fixedDeclaration(&mut self, _ctx: &FixedDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#fixedDeclaration}.
 * @param ctx the parse tree
 */
fn exit_fixedDeclaration(&mut self, _ctx: &FixedDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#enumDeclaration}.
 * @param ctx the parse tree
 */
fn enter_enumDeclaration(&mut self, _ctx: &EnumDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#enumDeclaration}.
 * @param ctx the parse tree
 */
fn exit_enumDeclaration(&mut self, _ctx: &EnumDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#enumSymbol}.
 * @param ctx the parse tree
 */
fn enter_enumSymbol(&mut self, _ctx: &EnumSymbolContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#enumSymbol}.
 * @param ctx the parse tree
 */
fn exit_enumSymbol(&mut self, _ctx: &EnumSymbolContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#enumDefault}.
 * @param ctx the parse tree
 */
fn enter_enumDefault(&mut self, _ctx: &EnumDefaultContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#enumDefault}.
 * @param ctx the parse tree
 */
fn exit_enumDefault(&mut self, _ctx: &EnumDefaultContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#recordDeclaration}.
 * @param ctx the parse tree
 */
fn enter_recordDeclaration(&mut self, _ctx: &RecordDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#recordDeclaration}.
 * @param ctx the parse tree
 */
fn exit_recordDeclaration(&mut self, _ctx: &RecordDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#recordBody}.
 * @param ctx the parse tree
 */
fn enter_recordBody(&mut self, _ctx: &RecordBodyContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#recordBody}.
 * @param ctx the parse tree
 */
fn exit_recordBody(&mut self, _ctx: &RecordBodyContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#fieldDeclaration}.
 * @param ctx the parse tree
 */
fn enter_fieldDeclaration(&mut self, _ctx: &FieldDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#fieldDeclaration}.
 * @param ctx the parse tree
 */
fn exit_fieldDeclaration(&mut self, _ctx: &FieldDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#variableDeclaration}.
 * @param ctx the parse tree
 */
fn enter_variableDeclaration(&mut self, _ctx: &VariableDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#variableDeclaration}.
 * @param ctx the parse tree
 */
fn exit_variableDeclaration(&mut self, _ctx: &VariableDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#messageDeclaration}.
 * @param ctx the parse tree
 */
fn enter_messageDeclaration(&mut self, _ctx: &MessageDeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#messageDeclaration}.
 * @param ctx the parse tree
 */
fn exit_messageDeclaration(&mut self, _ctx: &MessageDeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#formalParameter}.
 * @param ctx the parse tree
 */
fn enter_formalParameter(&mut self, _ctx: &FormalParameterContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#formalParameter}.
 * @param ctx the parse tree
 */
fn exit_formalParameter(&mut self, _ctx: &FormalParameterContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#resultType}.
 * @param ctx the parse tree
 */
fn enter_resultType(&mut self, _ctx: &ResultTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#resultType}.
 * @param ctx the parse tree
 */
fn exit_resultType(&mut self, _ctx: &ResultTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#fullType}.
 * @param ctx the parse tree
 */
fn enter_fullType(&mut self, _ctx: &FullTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#fullType}.
 * @param ctx the parse tree
 */
fn exit_fullType(&mut self, _ctx: &FullTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#plainType}.
 * @param ctx the parse tree
 */
fn enter_plainType(&mut self, _ctx: &PlainTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#plainType}.
 * @param ctx the parse tree
 */
fn exit_plainType(&mut self, _ctx: &PlainTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#nullableType}.
 * @param ctx the parse tree
 */
fn enter_nullableType(&mut self, _ctx: &NullableTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#nullableType}.
 * @param ctx the parse tree
 */
fn exit_nullableType(&mut self, _ctx: &NullableTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#primitiveType}.
 * @param ctx the parse tree
 */
fn enter_primitiveType(&mut self, _ctx: &PrimitiveTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#primitiveType}.
 * @param ctx the parse tree
 */
fn exit_primitiveType(&mut self, _ctx: &PrimitiveTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#arrayType}.
 * @param ctx the parse tree
 */
fn enter_arrayType(&mut self, _ctx: &ArrayTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#arrayType}.
 * @param ctx the parse tree
 */
fn exit_arrayType(&mut self, _ctx: &ArrayTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#mapType}.
 * @param ctx the parse tree
 */
fn enter_mapType(&mut self, _ctx: &MapTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#mapType}.
 * @param ctx the parse tree
 */
fn exit_mapType(&mut self, _ctx: &MapTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#unionType}.
 * @param ctx the parse tree
 */
fn enter_unionType(&mut self, _ctx: &UnionTypeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#unionType}.
 * @param ctx the parse tree
 */
fn exit_unionType(&mut self, _ctx: &UnionTypeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#jsonValue}.
 * @param ctx the parse tree
 */
fn enter_jsonValue(&mut self, _ctx: &JsonValueContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#jsonValue}.
 * @param ctx the parse tree
 */
fn exit_jsonValue(&mut self, _ctx: &JsonValueContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#jsonLiteral}.
 * @param ctx the parse tree
 */
fn enter_jsonLiteral(&mut self, _ctx: &JsonLiteralContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#jsonLiteral}.
 * @param ctx the parse tree
 */
fn exit_jsonLiteral(&mut self, _ctx: &JsonLiteralContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#jsonObject}.
 * @param ctx the parse tree
 */
fn enter_jsonObject(&mut self, _ctx: &JsonObjectContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#jsonObject}.
 * @param ctx the parse tree
 */
fn exit_jsonObject(&mut self, _ctx: &JsonObjectContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#jsonPair}.
 * @param ctx the parse tree
 */
fn enter_jsonPair(&mut self, _ctx: &JsonPairContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#jsonPair}.
 * @param ctx the parse tree
 */
fn exit_jsonPair(&mut self, _ctx: &JsonPairContext<'input>) { }
/**
 * Enter a parse tree produced by {@link IdlParser#jsonArray}.
 * @param ctx the parse tree
 */
fn enter_jsonArray(&mut self, _ctx: &JsonArrayContext<'input>) { }
/**
 * Exit a parse tree produced by {@link IdlParser#jsonArray}.
 * @param ctx the parse tree
 */
fn exit_jsonArray(&mut self, _ctx: &JsonArrayContext<'input>) { }

}

antlr4rust::coerce_from!{ 'input : IdlListener<'input> }


