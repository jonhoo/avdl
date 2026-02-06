// Generated from avro/share/idl_grammar/org/apache/avro/idl/Idl.g4 by ANTLR 4.13.2

use super::idlparser::*;
use antlr4rust::tree::ParseTreeListener;

// A complete Visitor for a parse tree produced by IdlParser.

pub trait IdlBaseListener<'input>:
    ParseTreeListener<'input, IdlParserContextType> {

    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_idlfile(&mut self, _ctx: &IdlFileContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_idlfile(&mut self, _ctx: &IdlFileContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_protocoldeclaration(&mut self, _ctx: &ProtocolDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_protocoldeclaration(&mut self, _ctx: &ProtocolDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_protocoldeclarationbody(&mut self, _ctx: &ProtocolDeclarationBodyContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_protocoldeclarationbody(&mut self, _ctx: &ProtocolDeclarationBodyContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_namespacedeclaration(&mut self, _ctx: &NamespaceDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_namespacedeclaration(&mut self, _ctx: &NamespaceDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_mainschemadeclaration(&mut self, _ctx: &MainSchemaDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_mainschemadeclaration(&mut self, _ctx: &MainSchemaDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_identifier(&mut self, _ctx: &IdentifierContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_identifier(&mut self, _ctx: &IdentifierContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_schemaproperty(&mut self, _ctx: &SchemaPropertyContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_schemaproperty(&mut self, _ctx: &SchemaPropertyContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_importstatement(&mut self, _ctx: &ImportStatementContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_importstatement(&mut self, _ctx: &ImportStatementContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_namedschemadeclaration(&mut self, _ctx: &NamedSchemaDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_namedschemadeclaration(&mut self, _ctx: &NamedSchemaDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_fixeddeclaration(&mut self, _ctx: &FixedDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_fixeddeclaration(&mut self, _ctx: &FixedDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_enumdeclaration(&mut self, _ctx: &EnumDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_enumdeclaration(&mut self, _ctx: &EnumDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_enumsymbol(&mut self, _ctx: &EnumSymbolContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_enumsymbol(&mut self, _ctx: &EnumSymbolContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_enumdefault(&mut self, _ctx: &EnumDefaultContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_enumdefault(&mut self, _ctx: &EnumDefaultContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_recorddeclaration(&mut self, _ctx: &RecordDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_recorddeclaration(&mut self, _ctx: &RecordDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_recordbody(&mut self, _ctx: &RecordBodyContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_recordbody(&mut self, _ctx: &RecordBodyContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_fielddeclaration(&mut self, _ctx: &FieldDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_fielddeclaration(&mut self, _ctx: &FieldDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_variabledeclaration(&mut self, _ctx: &VariableDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_variabledeclaration(&mut self, _ctx: &VariableDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_messagedeclaration(&mut self, _ctx: &MessageDeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_messagedeclaration(&mut self, _ctx: &MessageDeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_formalparameter(&mut self, _ctx: &FormalParameterContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_formalparameter(&mut self, _ctx: &FormalParameterContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_resulttype(&mut self, _ctx: &ResultTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_resulttype(&mut self, _ctx: &ResultTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_fulltype(&mut self, _ctx: &FullTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_fulltype(&mut self, _ctx: &FullTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_plaintype(&mut self, _ctx: &PlainTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_plaintype(&mut self, _ctx: &PlainTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_nullabletype(&mut self, _ctx: &NullableTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_nullabletype(&mut self, _ctx: &NullableTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_primitivetype(&mut self, _ctx: &PrimitiveTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_primitivetype(&mut self, _ctx: &PrimitiveTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_arraytype(&mut self, _ctx: &ArrayTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_arraytype(&mut self, _ctx: &ArrayTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_maptype(&mut self, _ctx: &MapTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_maptype(&mut self, _ctx: &MapTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_uniontype(&mut self, _ctx: &UnionTypeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_uniontype(&mut self, _ctx: &UnionTypeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonvalue(&mut self, _ctx: &JsonValueContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonvalue(&mut self, _ctx: &JsonValueContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonliteral(&mut self, _ctx: &JsonLiteralContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonliteral(&mut self, _ctx: &JsonLiteralContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonobject(&mut self, _ctx: &JsonObjectContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonobject(&mut self, _ctx: &JsonObjectContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonpair(&mut self, _ctx: &JsonPairContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonpair(&mut self, _ctx: &JsonPairContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonarray(&mut self, _ctx: &JsonArrayContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  IdlBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonarray(&mut self, _ctx: &JsonArrayContext<'input>) {}


}