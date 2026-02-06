// Generated from avro/share/idl_grammar/org/apache/avro/idl/Idl.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
use antlr4rust::PredictionContextCache;
use antlr4rust::parser::{Parser, BaseParser, ParserRecog, ParserNodeType};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::TokenSource;
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
use antlr4rust::errors::*;
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::recognizer::{Recognizer,Actions};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::error_strategy::{ErrorStrategy, DefaultErrorStrategy};
use antlr4rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext,cast,cast_mut};
use antlr4rust::tree::*;
use antlr4rust::token::{TOKEN_EOF,OwningToken,Token};
use antlr4rust::int_stream::EOF;
use antlr4rust::vocabulary::{Vocabulary,VocabularyImpl};
use antlr4rust::token_factory::{CommonTokenFactory,TokenFactory, TokenAware};
use super::idllistener::*;
use antlr4rust::lazy_static;
use antlr4rust::{TidAble,TidExt};

use std::marker::PhantomData;
use std::sync::Arc;
use std::rc::Rc;
use std::convert::TryFrom;
use std::cell::RefCell;
use std::ops::{DerefMut, Deref};
use std::borrow::{Borrow,BorrowMut};
use std::any::{Any,TypeId};

		pub const Idl_T__0:i32=1; 
		pub const Idl_DocComment:i32=2; 
		pub const Idl_EmptyComment:i32=3; 
		pub const Idl_MultiLineComment:i32=4; 
		pub const Idl_SingleLineComment:i32=5; 
		pub const Idl_WS:i32=6; 
		pub const Idl_Protocol:i32=7; 
		pub const Idl_Namespace:i32=8; 
		pub const Idl_Import:i32=9; 
		pub const Idl_IDL:i32=10; 
		pub const Idl_Schema:i32=11; 
		pub const Idl_Enum:i32=12; 
		pub const Idl_Fixed:i32=13; 
		pub const Idl_Error:i32=14; 
		pub const Idl_Record:i32=15; 
		pub const Idl_Array:i32=16; 
		pub const Idl_Map:i32=17; 
		pub const Idl_Union:i32=18; 
		pub const Idl_Boolean:i32=19; 
		pub const Idl_Int:i32=20; 
		pub const Idl_Long:i32=21; 
		pub const Idl_Float:i32=22; 
		pub const Idl_Double:i32=23; 
		pub const Idl_String:i32=24; 
		pub const Idl_Bytes:i32=25; 
		pub const Idl_Null:i32=26; 
		pub const Idl_BTrue:i32=27; 
		pub const Idl_BFalse:i32=28; 
		pub const Idl_Decimal:i32=29; 
		pub const Idl_Date:i32=30; 
		pub const Idl_Time:i32=31; 
		pub const Idl_Timestamp:i32=32; 
		pub const Idl_LocalTimestamp:i32=33; 
		pub const Idl_UUID:i32=34; 
		pub const Idl_Void:i32=35; 
		pub const Idl_Oneway:i32=36; 
		pub const Idl_Throws:i32=37; 
		pub const Idl_LParen:i32=38; 
		pub const Idl_RParen:i32=39; 
		pub const Idl_LBrace:i32=40; 
		pub const Idl_RBrace:i32=41; 
		pub const Idl_LBracket:i32=42; 
		pub const Idl_RBracket:i32=43; 
		pub const Idl_Colon:i32=44; 
		pub const Idl_Semicolon:i32=45; 
		pub const Idl_Comma:i32=46; 
		pub const Idl_At:i32=47; 
		pub const Idl_Equals:i32=48; 
		pub const Idl_Dot:i32=49; 
		pub const Idl_Dash:i32=50; 
		pub const Idl_QuestionMark:i32=51; 
		pub const Idl_LT:i32=52; 
		pub const Idl_GT:i32=53; 
		pub const Idl_StringLiteral:i32=54; 
		pub const Idl_IntegerLiteral:i32=55; 
		pub const Idl_FloatingPointLiteral:i32=56; 
		pub const Idl_IdentifierToken:i32=57;
	pub const Idl_EOF:i32=EOF;
	pub const RULE_idlFile:usize = 0; 
	pub const RULE_protocolDeclaration:usize = 1; 
	pub const RULE_protocolDeclarationBody:usize = 2; 
	pub const RULE_namespaceDeclaration:usize = 3; 
	pub const RULE_mainSchemaDeclaration:usize = 4; 
	pub const RULE_identifier:usize = 5; 
	pub const RULE_schemaProperty:usize = 6; 
	pub const RULE_importStatement:usize = 7; 
	pub const RULE_namedSchemaDeclaration:usize = 8; 
	pub const RULE_fixedDeclaration:usize = 9; 
	pub const RULE_enumDeclaration:usize = 10; 
	pub const RULE_enumSymbol:usize = 11; 
	pub const RULE_enumDefault:usize = 12; 
	pub const RULE_recordDeclaration:usize = 13; 
	pub const RULE_recordBody:usize = 14; 
	pub const RULE_fieldDeclaration:usize = 15; 
	pub const RULE_variableDeclaration:usize = 16; 
	pub const RULE_messageDeclaration:usize = 17; 
	pub const RULE_formalParameter:usize = 18; 
	pub const RULE_resultType:usize = 19; 
	pub const RULE_fullType:usize = 20; 
	pub const RULE_plainType:usize = 21; 
	pub const RULE_nullableType:usize = 22; 
	pub const RULE_primitiveType:usize = 23; 
	pub const RULE_arrayType:usize = 24; 
	pub const RULE_mapType:usize = 25; 
	pub const RULE_unionType:usize = 26; 
	pub const RULE_jsonValue:usize = 27; 
	pub const RULE_jsonLiteral:usize = 28; 
	pub const RULE_jsonObject:usize = 29; 
	pub const RULE_jsonPair:usize = 30; 
	pub const RULE_jsonArray:usize = 31;
	pub const ruleNames: [&'static str; 32] =  [
		"idlFile", "protocolDeclaration", "protocolDeclarationBody", "namespaceDeclaration", 
		"mainSchemaDeclaration", "identifier", "schemaProperty", "importStatement", 
		"namedSchemaDeclaration", "fixedDeclaration", "enumDeclaration", "enumSymbol", 
		"enumDefault", "recordDeclaration", "recordBody", "fieldDeclaration", 
		"variableDeclaration", "messageDeclaration", "formalParameter", "resultType", 
		"fullType", "plainType", "nullableType", "primitiveType", "arrayType", 
		"mapType", "unionType", "jsonValue", "jsonLiteral", "jsonObject", "jsonPair", 
		"jsonArray"
	];


	pub const _LITERAL_NAMES: [Option<&'static str>;54] = [
		None, Some("'\\u001A'"), None, Some("'/**/'"), None, None, None, Some("'protocol'"), 
		Some("'namespace'"), Some("'import'"), Some("'idl'"), Some("'schema'"), 
		Some("'enum'"), Some("'fixed'"), Some("'error'"), Some("'record'"), Some("'array'"), 
		Some("'map'"), Some("'union'"), Some("'boolean'"), Some("'int'"), Some("'long'"), 
		Some("'float'"), Some("'double'"), Some("'string'"), Some("'bytes'"), 
		Some("'null'"), Some("'true'"), Some("'false'"), Some("'decimal'"), Some("'date'"), 
		Some("'time_ms'"), Some("'timestamp_ms'"), Some("'local_timestamp_ms'"), 
		Some("'uuid'"), Some("'void'"), Some("'oneway'"), Some("'throws'"), Some("'('"), 
		Some("')'"), Some("'{'"), Some("'}'"), Some("'['"), Some("']'"), Some("':'"), 
		Some("';'"), Some("','"), Some("'@'"), Some("'='"), Some("'.'"), Some("'-'"), 
		Some("'?'"), Some("'<'"), Some("'>'")
	];
	pub const _SYMBOLIC_NAMES: [Option<&'static str>;58]  = [
		None, None, Some("DocComment"), Some("EmptyComment"), Some("MultiLineComment"), 
		Some("SingleLineComment"), Some("WS"), Some("Protocol"), Some("Namespace"), 
		Some("Import"), Some("IDL"), Some("Schema"), Some("Enum"), Some("Fixed"), 
		Some("Error"), Some("Record"), Some("Array"), Some("Map"), Some("Union"), 
		Some("Boolean"), Some("Int"), Some("Long"), Some("Float"), Some("Double"), 
		Some("String"), Some("Bytes"), Some("Null"), Some("BTrue"), Some("BFalse"), 
		Some("Decimal"), Some("Date"), Some("Time"), Some("Timestamp"), Some("LocalTimestamp"), 
		Some("UUID"), Some("Void"), Some("Oneway"), Some("Throws"), Some("LParen"), 
		Some("RParen"), Some("LBrace"), Some("RBrace"), Some("LBracket"), Some("RBracket"), 
		Some("Colon"), Some("Semicolon"), Some("Comma"), Some("At"), Some("Equals"), 
		Some("Dot"), Some("Dash"), Some("QuestionMark"), Some("LT"), Some("GT"), 
		Some("StringLiteral"), Some("IntegerLiteral"), Some("FloatingPointLiteral"), 
		Some("IdentifierToken")
	];
	lazy_static!{
	    static ref _shared_context_cache: Arc<PredictionContextCache> = Arc::new(PredictionContextCache::new());
		static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(_LITERAL_NAMES.iter(), _SYMBOLIC_NAMES.iter(), None));
	}


type BaseParserType<'input, I> =
	BaseParser<'input,IdlParserExt<'input>, I, IdlParserContextType , dyn IdlListener<'input> + 'input >;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type IdlTreeWalker<'input,'a> =
	ParseTreeWalker<'input, 'a, IdlParserContextType , dyn IdlListener<'input> + 'a>;

/// Parser for Idl grammar
pub struct IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	base:BaseParserType<'input,I>,
	interpreter:Arc<ParserATNSimulator>,
	_shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >,
}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn set_error_strategy(&mut self, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) {
        self.err_handler = strategy
    }

    pub fn with_strategy(input: I, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) -> Self {
		antlr4rust::recognizer::check_version("0","5");
		let interpreter = Arc::new(ParserATNSimulator::new(
			_ATN.clone(),
			_decision_to_DFA.clone(),
			_shared_context_cache.clone(),
		));
		Self {
			base: BaseParser::new_base_parser(
				input,
				Arc::clone(&interpreter),
				IdlParserExt{
					_pd: Default::default(),
				}
			),
			interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }

}

type DynStrategy<'input,I> = Box<dyn ErrorStrategy<'input,BaseParserType<'input,I>> + 'input>;

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn new(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for IdlParser
pub trait IdlParserContext<'input>:
	for<'x> Listenable<dyn IdlListener<'input> + 'x > + 
	ParserRuleContext<'input, TF=LocalTokenFactory<'input>, Ctx=IdlParserContextType>
{}

antlr4rust::coerce_from!{ 'input : IdlParserContext<'input> }

impl<'input> IdlParserContext<'input> for TerminalNode<'input,IdlParserContextType> {}
impl<'input> IdlParserContext<'input> for ErrorNode<'input,IdlParserContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn IdlParserContext<'input> + 'input }

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn IdlListener<'input> + 'input }

pub struct IdlParserContextType;
antlr4rust::tid!{IdlParserContextType}

impl<'input> ParserNodeType<'input> for IdlParserContextType{
	type TF = LocalTokenFactory<'input>;
	type Type = dyn IdlParserContext<'input> + 'input;
}

impl<'input, I> Deref for IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    type Target = BaseParserType<'input,I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct IdlParserExt<'input>{
	_pd: PhantomData<&'input str>,
}

impl<'input> IdlParserExt<'input>{
}
antlr4rust::tid! { IdlParserExt<'a> }

impl<'input> TokenAware<'input> for IdlParserExt<'input>{
	type TF = LocalTokenFactory<'input>;
}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> ParserRecog<'input, BaseParserType<'input,I>> for IdlParserExt<'input>{}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> Actions<'input, BaseParserType<'input,I>> for IdlParserExt<'input>{
	fn get_grammar_file_name(&self) -> & str{ "Idl.g4"}

   	fn get_rule_names(&self) -> &[& str] {&ruleNames}

   	fn get_vocabulary(&self) -> &dyn Vocabulary { &**VOCABULARY }
}
//------------------- idlFile ----------------
pub type IdlFileContextAll<'input> = IdlFileContext<'input>;


pub type IdlFileContext<'input> = BaseParserRuleContext<'input,IdlFileContextExt<'input>>;

#[derive(Clone)]
pub struct IdlFileContextExt<'input>{
	pub protocol: Option<Rc<ProtocolDeclarationContextAll<'input>>>,
	pub namespace: Option<Rc<NamespaceDeclarationContextAll<'input>>>,
	pub mainSchema: Option<Rc<MainSchemaDeclarationContextAll<'input>>>,
	pub importStatement: Option<Rc<ImportStatementContextAll<'input>>>,
	pub imports:Vec<Rc<ImportStatementContextAll<'input>>>,
	pub namedSchemaDeclaration: Option<Rc<NamedSchemaDeclarationContextAll<'input>>>,
	pub namedSchemas:Vec<Rc<NamedSchemaDeclarationContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for IdlFileContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for IdlFileContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_idlFile(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_idlFile(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for IdlFileContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_idlFile }
	//fn type_rule_index() -> usize where Self: Sized { RULE_idlFile }
}
antlr4rust::tid!{IdlFileContextExt<'a>}

impl<'input> IdlFileContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<IdlFileContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,IdlFileContextExt{
				protocol: None, namespace: None, mainSchema: None, importStatement: None, namedSchemaDeclaration: None, 
				imports: Vec::new(), namedSchemas: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait IdlFileContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<IdlFileContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_EOF, 0)
}
fn protocolDeclaration(&self) -> Option<Rc<ProtocolDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn namespaceDeclaration(&self) -> Option<Rc<NamespaceDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn mainSchemaDeclaration(&self) -> Option<Rc<MainSchemaDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn importStatement_all(&self) ->  Vec<Rc<ImportStatementContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn importStatement(&self, i: usize) -> Option<Rc<ImportStatementContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn namedSchemaDeclaration_all(&self) ->  Vec<Rc<NamedSchemaDeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn namedSchemaDeclaration(&self, i: usize) -> Option<Rc<NamedSchemaDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> IdlFileContextAttrs<'input> for IdlFileContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn idlFile(&mut self,)
	-> Result<Rc<IdlFileContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = IdlFileContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_idlFile);
        let mut _localctx: Rc<IdlFileContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(78);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(4,&mut recog.base)? {
				1 =>{
					{
					/*InvokeRule protocolDeclaration*/
					recog.base.set_state(64);
					let tmp = recog.protocolDeclaration()?;
					 cast_mut::<_,IdlFileContext >(&mut _localctx).protocol = Some(tmp.clone());
					  

					}
				}
			,
				2 =>{
					{
					recog.base.set_state(66);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if _la==Idl_Namespace {
						{
						/*InvokeRule namespaceDeclaration*/
						recog.base.set_state(65);
						let tmp = recog.namespaceDeclaration()?;
						 cast_mut::<_,IdlFileContext >(&mut _localctx).namespace = Some(tmp.clone());
						  

						}
					}

					recog.base.set_state(69);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if _la==Idl_Schema {
						{
						/*InvokeRule mainSchemaDeclaration*/
						recog.base.set_state(68);
						let tmp = recog.mainSchemaDeclaration()?;
						 cast_mut::<_,IdlFileContext >(&mut _localctx).mainSchema = Some(tmp.clone());
						  

						}
					}

					recog.base.set_state(75);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while (((_la) & !0x3f) == 0 && ((1usize << _la) & 61956) != 0) || _la==Idl_At {
						{
						recog.base.set_state(73);
						recog.err_handler.sync(&mut recog.base)?;
						match recog.base.input.la(1) {
						Idl_Import 
							=> {
								{
								/*InvokeRule importStatement*/
								recog.base.set_state(71);
								let tmp = recog.importStatement()?;
								 cast_mut::<_,IdlFileContext >(&mut _localctx).importStatement = Some(tmp.clone());
								  

								let temp =  cast_mut::<_,IdlFileContext >(&mut _localctx).importStatement.clone().unwrap()
								 ;
								 cast_mut::<_,IdlFileContext >(&mut _localctx).imports.push(temp);
								  
								}
							}

						Idl_DocComment |Idl_Enum |Idl_Fixed |Idl_Error |Idl_Record |Idl_At 
							=> {
								{
								/*InvokeRule namedSchemaDeclaration*/
								recog.base.set_state(72);
								let tmp = recog.namedSchemaDeclaration()?;
								 cast_mut::<_,IdlFileContext >(&mut _localctx).namedSchemaDeclaration = Some(tmp.clone());
								  

								let temp =  cast_mut::<_,IdlFileContext >(&mut _localctx).namedSchemaDeclaration.clone().unwrap()
								 ;
								 cast_mut::<_,IdlFileContext >(&mut _localctx).namedSchemas.push(temp);
								  
								}
							}

							_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
						}
						}
						recog.base.set_state(77);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					}
				}

				_ => {}
			}
			recog.base.set_state(87);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_T__0 {
				{
				recog.base.set_state(80);
				recog.base.match_token(Idl_T__0,&mut recog.err_handler)?;

				recog.base.set_state(84);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(5,&mut recog.base)?;
				while { _alt!=1 && _alt!=INVALID_ALT } {
					if _alt==1+1 {
						{
						{
						recog.base.set_state(81);
						recog.base.match_wildcard(&mut recog.err_handler)?;

						}
						} 
					}
					recog.base.set_state(86);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(5,&mut recog.base)?;
				}
				}
			}

			recog.base.set_state(89);
			recog.base.match_token(Idl_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- protocolDeclaration ----------------
pub type ProtocolDeclarationContextAll<'input> = ProtocolDeclarationContext<'input>;


pub type ProtocolDeclarationContext<'input> = BaseParserRuleContext<'input,ProtocolDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct ProtocolDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
	pub body: Option<Rc<ProtocolDeclarationBodyContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for ProtocolDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for ProtocolDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_protocolDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_protocolDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for ProtocolDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_protocolDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_protocolDeclaration }
}
antlr4rust::tid!{ProtocolDeclarationContextExt<'a>}

impl<'input> ProtocolDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ProtocolDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ProtocolDeclarationContextExt{
				doc: None, 
				schemaProperty: None, name: None, body: None, 
				schemaProperties: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait ProtocolDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<ProtocolDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Protocol
/// Returns `None` if there is no child corresponding to token Protocol
fn Protocol(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Protocol, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn protocolDeclarationBody(&self) -> Option<Rc<ProtocolDeclarationBodyContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ProtocolDeclarationContextAttrs<'input> for ProtocolDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn protocolDeclaration(&mut self,)
	-> Result<Rc<ProtocolDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ProtocolDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_protocolDeclaration);
        let mut _localctx: Rc<ProtocolDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(92);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(91);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,ProtocolDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(97);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(94);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,ProtocolDeclarationContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,ProtocolDeclarationContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,ProtocolDeclarationContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(99);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(100);
			recog.base.match_token(Idl_Protocol,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(101);
			let tmp = recog.identifier()?;
			 cast_mut::<_,ProtocolDeclarationContext >(&mut _localctx).name = Some(tmp.clone());
			  

			/*InvokeRule protocolDeclarationBody*/
			recog.base.set_state(102);
			let tmp = recog.protocolDeclarationBody()?;
			 cast_mut::<_,ProtocolDeclarationContext >(&mut _localctx).body = Some(tmp.clone());
			  

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- protocolDeclarationBody ----------------
pub type ProtocolDeclarationBodyContextAll<'input> = ProtocolDeclarationBodyContext<'input>;


pub type ProtocolDeclarationBodyContext<'input> = BaseParserRuleContext<'input,ProtocolDeclarationBodyContextExt<'input>>;

#[derive(Clone)]
pub struct ProtocolDeclarationBodyContextExt<'input>{
	pub importStatement: Option<Rc<ImportStatementContextAll<'input>>>,
	pub imports:Vec<Rc<ImportStatementContextAll<'input>>>,
	pub namedSchemaDeclaration: Option<Rc<NamedSchemaDeclarationContextAll<'input>>>,
	pub namedSchemas:Vec<Rc<NamedSchemaDeclarationContextAll<'input>>>,
	pub messageDeclaration: Option<Rc<MessageDeclarationContextAll<'input>>>,
	pub messages:Vec<Rc<MessageDeclarationContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for ProtocolDeclarationBodyContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for ProtocolDeclarationBodyContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_protocolDeclarationBody(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_protocolDeclarationBody(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for ProtocolDeclarationBodyContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_protocolDeclarationBody }
	//fn type_rule_index() -> usize where Self: Sized { RULE_protocolDeclarationBody }
}
antlr4rust::tid!{ProtocolDeclarationBodyContextExt<'a>}

impl<'input> ProtocolDeclarationBodyContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ProtocolDeclarationBodyContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ProtocolDeclarationBodyContextExt{
				importStatement: None, namedSchemaDeclaration: None, messageDeclaration: None, 
				imports: Vec::new(), namedSchemas: Vec::new(), messages: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait ProtocolDeclarationBodyContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<ProtocolDeclarationBodyContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token LBrace
/// Returns `None` if there is no child corresponding to token LBrace
fn LBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LBrace, 0)
}
/// Retrieves first TerminalNode corresponding to token RBrace
/// Returns `None` if there is no child corresponding to token RBrace
fn RBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RBrace, 0)
}
fn importStatement_all(&self) ->  Vec<Rc<ImportStatementContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn importStatement(&self, i: usize) -> Option<Rc<ImportStatementContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn namedSchemaDeclaration_all(&self) ->  Vec<Rc<NamedSchemaDeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn namedSchemaDeclaration(&self, i: usize) -> Option<Rc<NamedSchemaDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn messageDeclaration_all(&self) ->  Vec<Rc<MessageDeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn messageDeclaration(&self, i: usize) -> Option<Rc<MessageDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ProtocolDeclarationBodyContextAttrs<'input> for ProtocolDeclarationBodyContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn protocolDeclarationBody(&mut self,)
	-> Result<Rc<ProtocolDeclarationBodyContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ProtocolDeclarationBodyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_protocolDeclarationBody);
        let mut _localctx: Rc<ProtocolDeclarationBodyContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(104);
			recog.base.match_token(Idl_LBrace,&mut recog.err_handler)?;

			recog.base.set_state(110);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 4294967172) != 0) || ((((_la - 32)) & !0x3f) == 0 && ((1usize << (_la - 32)) & 33587263) != 0) {
				{
				recog.base.set_state(108);
				recog.err_handler.sync(&mut recog.base)?;
				match  recog.interpreter.adaptive_predict(9,&mut recog.base)? {
					1 =>{
						{
						/*InvokeRule importStatement*/
						recog.base.set_state(105);
						let tmp = recog.importStatement()?;
						 cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).importStatement = Some(tmp.clone());
						  

						let temp =  cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).importStatement.clone().unwrap()
						 ;
						 cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).imports.push(temp);
						  
						}
					}
				,
					2 =>{
						{
						/*InvokeRule namedSchemaDeclaration*/
						recog.base.set_state(106);
						let tmp = recog.namedSchemaDeclaration()?;
						 cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).namedSchemaDeclaration = Some(tmp.clone());
						  

						let temp =  cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).namedSchemaDeclaration.clone().unwrap()
						 ;
						 cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).namedSchemas.push(temp);
						  
						}
					}
				,
					3 =>{
						{
						/*InvokeRule messageDeclaration*/
						recog.base.set_state(107);
						let tmp = recog.messageDeclaration()?;
						 cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).messageDeclaration = Some(tmp.clone());
						  

						let temp =  cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).messageDeclaration.clone().unwrap()
						 ;
						 cast_mut::<_,ProtocolDeclarationBodyContext >(&mut _localctx).messages.push(temp);
						  
						}
					}

					_ => {}
				}
				}
				recog.base.set_state(112);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(113);
			recog.base.match_token(Idl_RBrace,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- namespaceDeclaration ----------------
pub type NamespaceDeclarationContextAll<'input> = NamespaceDeclarationContext<'input>;


pub type NamespaceDeclarationContext<'input> = BaseParserRuleContext<'input,NamespaceDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct NamespaceDeclarationContextExt<'input>{
	pub namespace: Option<Rc<IdentifierContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for NamespaceDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for NamespaceDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_namespaceDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_namespaceDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for NamespaceDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_namespaceDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_namespaceDeclaration }
}
antlr4rust::tid!{NamespaceDeclarationContextExt<'a>}

impl<'input> NamespaceDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<NamespaceDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,NamespaceDeclarationContextExt{
				namespace: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait NamespaceDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<NamespaceDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Namespace
/// Returns `None` if there is no child corresponding to token Namespace
fn Namespace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Namespace, 0)
}
/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> NamespaceDeclarationContextAttrs<'input> for NamespaceDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn namespaceDeclaration(&mut self,)
	-> Result<Rc<NamespaceDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = NamespaceDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_namespaceDeclaration);
        let mut _localctx: Rc<NamespaceDeclarationContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(115);
			recog.base.match_token(Idl_Namespace,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(116);
			let tmp = recog.identifier()?;
			 cast_mut::<_,NamespaceDeclarationContext >(&mut _localctx).namespace = Some(tmp.clone());
			  

			recog.base.set_state(117);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- mainSchemaDeclaration ----------------
pub type MainSchemaDeclarationContextAll<'input> = MainSchemaDeclarationContext<'input>;


pub type MainSchemaDeclarationContext<'input> = BaseParserRuleContext<'input,MainSchemaDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct MainSchemaDeclarationContextExt<'input>{
	pub mainSchema: Option<Rc<FullTypeContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for MainSchemaDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for MainSchemaDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_mainSchemaDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_mainSchemaDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for MainSchemaDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_mainSchemaDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_mainSchemaDeclaration }
}
antlr4rust::tid!{MainSchemaDeclarationContextExt<'a>}

impl<'input> MainSchemaDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<MainSchemaDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MainSchemaDeclarationContextExt{
				mainSchema: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait MainSchemaDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<MainSchemaDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Schema
/// Returns `None` if there is no child corresponding to token Schema
fn Schema(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Schema, 0)
}
/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
fn fullType(&self) -> Option<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> MainSchemaDeclarationContextAttrs<'input> for MainSchemaDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn mainSchemaDeclaration(&mut self,)
	-> Result<Rc<MainSchemaDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MainSchemaDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 8, RULE_mainSchemaDeclaration);
        let mut _localctx: Rc<MainSchemaDeclarationContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(119);
			recog.base.match_token(Idl_Schema,&mut recog.err_handler)?;

			/*InvokeRule fullType*/
			recog.base.set_state(120);
			let tmp = recog.fullType()?;
			 cast_mut::<_,MainSchemaDeclarationContext >(&mut _localctx).mainSchema = Some(tmp.clone());
			  

			recog.base.set_state(121);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- identifier ----------------
pub type IdentifierContextAll<'input> = IdentifierContext<'input>;


pub type IdentifierContext<'input> = BaseParserRuleContext<'input,IdentifierContextExt<'input>>;

#[derive(Clone)]
pub struct IdentifierContextExt<'input>{
	pub word: Option<TokenType<'input>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for IdentifierContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for IdentifierContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_identifier(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_identifier(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for IdentifierContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_identifier }
	//fn type_rule_index() -> usize where Self: Sized { RULE_identifier }
}
antlr4rust::tid!{IdentifierContextExt<'a>}

impl<'input> IdentifierContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<IdentifierContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,IdentifierContextExt{
				word: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait IdentifierContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<IdentifierContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token IdentifierToken
/// Returns `None` if there is no child corresponding to token IdentifierToken
fn IdentifierToken(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_IdentifierToken, 0)
}
/// Retrieves first TerminalNode corresponding to token Protocol
/// Returns `None` if there is no child corresponding to token Protocol
fn Protocol(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Protocol, 0)
}
/// Retrieves first TerminalNode corresponding to token Namespace
/// Returns `None` if there is no child corresponding to token Namespace
fn Namespace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Namespace, 0)
}
/// Retrieves first TerminalNode corresponding to token Import
/// Returns `None` if there is no child corresponding to token Import
fn Import(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Import, 0)
}
/// Retrieves first TerminalNode corresponding to token IDL
/// Returns `None` if there is no child corresponding to token IDL
fn IDL(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_IDL, 0)
}
/// Retrieves first TerminalNode corresponding to token Schema
/// Returns `None` if there is no child corresponding to token Schema
fn Schema(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Schema, 0)
}
/// Retrieves first TerminalNode corresponding to token Enum
/// Returns `None` if there is no child corresponding to token Enum
fn Enum(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Enum, 0)
}
/// Retrieves first TerminalNode corresponding to token Fixed
/// Returns `None` if there is no child corresponding to token Fixed
fn Fixed(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Fixed, 0)
}
/// Retrieves first TerminalNode corresponding to token Error
/// Returns `None` if there is no child corresponding to token Error
fn Error(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Error, 0)
}
/// Retrieves first TerminalNode corresponding to token Record
/// Returns `None` if there is no child corresponding to token Record
fn Record(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Record, 0)
}
/// Retrieves first TerminalNode corresponding to token Array
/// Returns `None` if there is no child corresponding to token Array
fn Array(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Array, 0)
}
/// Retrieves first TerminalNode corresponding to token Map
/// Returns `None` if there is no child corresponding to token Map
fn Map(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Map, 0)
}
/// Retrieves first TerminalNode corresponding to token Union
/// Returns `None` if there is no child corresponding to token Union
fn Union(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Union, 0)
}
/// Retrieves first TerminalNode corresponding to token Boolean
/// Returns `None` if there is no child corresponding to token Boolean
fn Boolean(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Boolean, 0)
}
/// Retrieves first TerminalNode corresponding to token Int
/// Returns `None` if there is no child corresponding to token Int
fn Int(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Int, 0)
}
/// Retrieves first TerminalNode corresponding to token Long
/// Returns `None` if there is no child corresponding to token Long
fn Long(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Long, 0)
}
/// Retrieves first TerminalNode corresponding to token Float
/// Returns `None` if there is no child corresponding to token Float
fn Float(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Float, 0)
}
/// Retrieves first TerminalNode corresponding to token Double
/// Returns `None` if there is no child corresponding to token Double
fn Double(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Double, 0)
}
/// Retrieves first TerminalNode corresponding to token String
/// Returns `None` if there is no child corresponding to token String
fn String(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_String, 0)
}
/// Retrieves first TerminalNode corresponding to token Bytes
/// Returns `None` if there is no child corresponding to token Bytes
fn Bytes(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Bytes, 0)
}
/// Retrieves first TerminalNode corresponding to token Null
/// Returns `None` if there is no child corresponding to token Null
fn Null(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Null, 0)
}
/// Retrieves first TerminalNode corresponding to token BTrue
/// Returns `None` if there is no child corresponding to token BTrue
fn BTrue(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_BTrue, 0)
}
/// Retrieves first TerminalNode corresponding to token BFalse
/// Returns `None` if there is no child corresponding to token BFalse
fn BFalse(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_BFalse, 0)
}
/// Retrieves first TerminalNode corresponding to token Decimal
/// Returns `None` if there is no child corresponding to token Decimal
fn Decimal(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Decimal, 0)
}
/// Retrieves first TerminalNode corresponding to token Date
/// Returns `None` if there is no child corresponding to token Date
fn Date(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Date, 0)
}
/// Retrieves first TerminalNode corresponding to token Time
/// Returns `None` if there is no child corresponding to token Time
fn Time(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Time, 0)
}
/// Retrieves first TerminalNode corresponding to token Timestamp
/// Returns `None` if there is no child corresponding to token Timestamp
fn Timestamp(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Timestamp, 0)
}
/// Retrieves first TerminalNode corresponding to token LocalTimestamp
/// Returns `None` if there is no child corresponding to token LocalTimestamp
fn LocalTimestamp(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LocalTimestamp, 0)
}
/// Retrieves first TerminalNode corresponding to token UUID
/// Returns `None` if there is no child corresponding to token UUID
fn UUID(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_UUID, 0)
}
/// Retrieves first TerminalNode corresponding to token Void
/// Returns `None` if there is no child corresponding to token Void
fn Void(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Void, 0)
}
/// Retrieves first TerminalNode corresponding to token Oneway
/// Returns `None` if there is no child corresponding to token Oneway
fn Oneway(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Oneway, 0)
}
/// Retrieves first TerminalNode corresponding to token Throws
/// Returns `None` if there is no child corresponding to token Throws
fn Throws(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Throws, 0)
}

}

impl<'input> IdentifierContextAttrs<'input> for IdentifierContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn identifier(&mut self,)
	-> Result<Rc<IdentifierContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = IdentifierContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 10, RULE_identifier);
        let mut _localctx: Rc<IdentifierContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(123);
			 cast_mut::<_,IdentifierContext >(&mut _localctx).word = recog.base.input.lt(1).cloned();
			 
			_la = recog.base.input.la(1);
			if { !((((_la) & !0x3f) == 0 && ((1usize << _la) & 4294967168) != 0) || ((((_la - 32)) & !0x3f) == 0 && ((1usize << (_la - 32)) & 33554495) != 0)) } {
				let tmp = recog.err_handler.recover_inline(&mut recog.base)?;
				 cast_mut::<_,IdentifierContext >(&mut _localctx).word = Some(tmp.clone());
				  

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- schemaProperty ----------------
pub type SchemaPropertyContextAll<'input> = SchemaPropertyContext<'input>;


pub type SchemaPropertyContext<'input> = BaseParserRuleContext<'input,SchemaPropertyContextExt<'input>>;

#[derive(Clone)]
pub struct SchemaPropertyContextExt<'input>{
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
	pub value: Option<Rc<JsonValueContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for SchemaPropertyContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for SchemaPropertyContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_schemaProperty(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_schemaProperty(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for SchemaPropertyContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_schemaProperty }
	//fn type_rule_index() -> usize where Self: Sized { RULE_schemaProperty }
}
antlr4rust::tid!{SchemaPropertyContextExt<'a>}

impl<'input> SchemaPropertyContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<SchemaPropertyContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,SchemaPropertyContextExt{
				name: None, value: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait SchemaPropertyContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<SchemaPropertyContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token At
/// Returns `None` if there is no child corresponding to token At
fn At(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_At, 0)
}
/// Retrieves first TerminalNode corresponding to token LParen
/// Returns `None` if there is no child corresponding to token LParen
fn LParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LParen, 0)
}
/// Retrieves first TerminalNode corresponding to token RParen
/// Returns `None` if there is no child corresponding to token RParen
fn RParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RParen, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> SchemaPropertyContextAttrs<'input> for SchemaPropertyContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn schemaProperty(&mut self,)
	-> Result<Rc<SchemaPropertyContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = SchemaPropertyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 12, RULE_schemaProperty);
        let mut _localctx: Rc<SchemaPropertyContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(125);
			recog.base.match_token(Idl_At,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(126);
			let tmp = recog.identifier()?;
			 cast_mut::<_,SchemaPropertyContext >(&mut _localctx).name = Some(tmp.clone());
			  

			recog.base.set_state(127);
			recog.base.match_token(Idl_LParen,&mut recog.err_handler)?;

			/*InvokeRule jsonValue*/
			recog.base.set_state(128);
			let tmp = recog.jsonValue()?;
			 cast_mut::<_,SchemaPropertyContext >(&mut _localctx).value = Some(tmp.clone());
			  

			recog.base.set_state(129);
			recog.base.match_token(Idl_RParen,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- importStatement ----------------
pub type ImportStatementContextAll<'input> = ImportStatementContext<'input>;


pub type ImportStatementContext<'input> = BaseParserRuleContext<'input,ImportStatementContextExt<'input>>;

#[derive(Clone)]
pub struct ImportStatementContextExt<'input>{
	pub importType: Option<TokenType<'input>>,
	pub location: Option<TokenType<'input>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for ImportStatementContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for ImportStatementContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_importStatement(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_importStatement(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for ImportStatementContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_importStatement }
	//fn type_rule_index() -> usize where Self: Sized { RULE_importStatement }
}
antlr4rust::tid!{ImportStatementContextExt<'a>}

impl<'input> ImportStatementContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ImportStatementContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ImportStatementContextExt{
				importType: None, location: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait ImportStatementContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<ImportStatementContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Import
/// Returns `None` if there is no child corresponding to token Import
fn Import(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Import, 0)
}
/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
/// Retrieves first TerminalNode corresponding to token StringLiteral
/// Returns `None` if there is no child corresponding to token StringLiteral
fn StringLiteral(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_StringLiteral, 0)
}
/// Retrieves first TerminalNode corresponding to token Schema
/// Returns `None` if there is no child corresponding to token Schema
fn Schema(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Schema, 0)
}
/// Retrieves first TerminalNode corresponding to token Protocol
/// Returns `None` if there is no child corresponding to token Protocol
fn Protocol(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Protocol, 0)
}
/// Retrieves first TerminalNode corresponding to token IDL
/// Returns `None` if there is no child corresponding to token IDL
fn IDL(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_IDL, 0)
}

}

impl<'input> ImportStatementContextAttrs<'input> for ImportStatementContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn importStatement(&mut self,)
	-> Result<Rc<ImportStatementContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ImportStatementContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 14, RULE_importStatement);
        let mut _localctx: Rc<ImportStatementContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(131);
			recog.base.match_token(Idl_Import,&mut recog.err_handler)?;

			recog.base.set_state(132);
			 cast_mut::<_,ImportStatementContext >(&mut _localctx).importType = recog.base.input.lt(1).cloned();
			 
			_la = recog.base.input.la(1);
			if { !((((_la) & !0x3f) == 0 && ((1usize << _la) & 3200) != 0)) } {
				let tmp = recog.err_handler.recover_inline(&mut recog.base)?;
				 cast_mut::<_,ImportStatementContext >(&mut _localctx).importType = Some(tmp.clone());
				  

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			recog.base.set_state(133);
			let tmp = recog.base.match_token(Idl_StringLiteral,&mut recog.err_handler)?;
			 cast_mut::<_,ImportStatementContext >(&mut _localctx).location = Some(tmp.clone());
			  

			recog.base.set_state(134);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- namedSchemaDeclaration ----------------
pub type NamedSchemaDeclarationContextAll<'input> = NamedSchemaDeclarationContext<'input>;


pub type NamedSchemaDeclarationContext<'input> = BaseParserRuleContext<'input,NamedSchemaDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct NamedSchemaDeclarationContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for NamedSchemaDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for NamedSchemaDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_namedSchemaDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_namedSchemaDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for NamedSchemaDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_namedSchemaDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_namedSchemaDeclaration }
}
antlr4rust::tid!{NamedSchemaDeclarationContextExt<'a>}

impl<'input> NamedSchemaDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<NamedSchemaDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,NamedSchemaDeclarationContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait NamedSchemaDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<NamedSchemaDeclarationContextExt<'input>>{

fn fixedDeclaration(&self) -> Option<Rc<FixedDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn enumDeclaration(&self) -> Option<Rc<EnumDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn recordDeclaration(&self) -> Option<Rc<RecordDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> NamedSchemaDeclarationContextAttrs<'input> for NamedSchemaDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn namedSchemaDeclaration(&mut self,)
	-> Result<Rc<NamedSchemaDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = NamedSchemaDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 16, RULE_namedSchemaDeclaration);
        let mut _localctx: Rc<NamedSchemaDeclarationContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(139);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(11,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule fixedDeclaration*/
					recog.base.set_state(136);
					recog.fixedDeclaration()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule enumDeclaration*/
					recog.base.set_state(137);
					recog.enumDeclaration()?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule recordDeclaration*/
					recog.base.set_state(138);
					recog.recordDeclaration()?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- fixedDeclaration ----------------
pub type FixedDeclarationContextAll<'input> = FixedDeclarationContext<'input>;


pub type FixedDeclarationContext<'input> = BaseParserRuleContext<'input,FixedDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct FixedDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
	pub size: Option<TokenType<'input>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for FixedDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for FixedDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_fixedDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_fixedDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for FixedDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_fixedDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_fixedDeclaration }
}
antlr4rust::tid!{FixedDeclarationContextExt<'a>}

impl<'input> FixedDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<FixedDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,FixedDeclarationContextExt{
				doc: None, size: None, 
				schemaProperty: None, name: None, 
				schemaProperties: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait FixedDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<FixedDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Fixed
/// Returns `None` if there is no child corresponding to token Fixed
fn Fixed(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Fixed, 0)
}
/// Retrieves first TerminalNode corresponding to token LParen
/// Returns `None` if there is no child corresponding to token LParen
fn LParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LParen, 0)
}
/// Retrieves first TerminalNode corresponding to token RParen
/// Returns `None` if there is no child corresponding to token RParen
fn RParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RParen, 0)
}
/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token IntegerLiteral
/// Returns `None` if there is no child corresponding to token IntegerLiteral
fn IntegerLiteral(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_IntegerLiteral, 0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> FixedDeclarationContextAttrs<'input> for FixedDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn fixedDeclaration(&mut self,)
	-> Result<Rc<FixedDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = FixedDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_fixedDeclaration);
        let mut _localctx: Rc<FixedDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(142);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(141);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,FixedDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(147);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(144);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,FixedDeclarationContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,FixedDeclarationContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,FixedDeclarationContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(149);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(150);
			recog.base.match_token(Idl_Fixed,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(151);
			let tmp = recog.identifier()?;
			 cast_mut::<_,FixedDeclarationContext >(&mut _localctx).name = Some(tmp.clone());
			  

			recog.base.set_state(152);
			recog.base.match_token(Idl_LParen,&mut recog.err_handler)?;

			recog.base.set_state(153);
			let tmp = recog.base.match_token(Idl_IntegerLiteral,&mut recog.err_handler)?;
			 cast_mut::<_,FixedDeclarationContext >(&mut _localctx).size = Some(tmp.clone());
			  

			recog.base.set_state(154);
			recog.base.match_token(Idl_RParen,&mut recog.err_handler)?;

			recog.base.set_state(155);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- enumDeclaration ----------------
pub type EnumDeclarationContextAll<'input> = EnumDeclarationContext<'input>;


pub type EnumDeclarationContext<'input> = BaseParserRuleContext<'input,EnumDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct EnumDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
	pub enumSymbol: Option<Rc<EnumSymbolContextAll<'input>>>,
	pub enumSymbols:Vec<Rc<EnumSymbolContextAll<'input>>>,
	pub defaultSymbol: Option<Rc<EnumDefaultContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for EnumDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for EnumDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_enumDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_enumDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for EnumDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_enumDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_enumDeclaration }
}
antlr4rust::tid!{EnumDeclarationContextExt<'a>}

impl<'input> EnumDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EnumDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EnumDeclarationContextExt{
				doc: None, 
				schemaProperty: None, name: None, enumSymbol: None, defaultSymbol: None, 
				schemaProperties: Vec::new(), enumSymbols: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait EnumDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<EnumDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Enum
/// Returns `None` if there is no child corresponding to token Enum
fn Enum(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Enum, 0)
}
/// Retrieves first TerminalNode corresponding to token LBrace
/// Returns `None` if there is no child corresponding to token LBrace
fn LBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LBrace, 0)
}
/// Retrieves first TerminalNode corresponding to token RBrace
/// Returns `None` if there is no child corresponding to token RBrace
fn RBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RBrace, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn enumSymbol_all(&self) ->  Vec<Rc<EnumSymbolContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn enumSymbol(&self, i: usize) -> Option<Rc<EnumSymbolContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn enumDefault(&self) -> Option<Rc<EnumDefaultContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token Comma in current rule
fn Comma_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token Comma, starting from 0.
/// Returns `None` if number of children corresponding to token Comma is less or equal than `i`.
fn Comma(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, i)
}

}

impl<'input> EnumDeclarationContextAttrs<'input> for EnumDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn enumDeclaration(&mut self,)
	-> Result<Rc<EnumDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EnumDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 20, RULE_enumDeclaration);
        let mut _localctx: Rc<EnumDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(158);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(157);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(163);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(160);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,EnumDeclarationContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(165);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(166);
			recog.base.match_token(Idl_Enum,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(167);
			let tmp = recog.identifier()?;
			 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).name = Some(tmp.clone());
			  

			recog.base.set_state(168);
			recog.base.match_token(Idl_LBrace,&mut recog.err_handler)?;

			recog.base.set_state(177);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 4294967172) != 0) || ((((_la - 32)) & !0x3f) == 0 && ((1usize << (_la - 32)) & 33587263) != 0) {
				{
				/*InvokeRule enumSymbol*/
				recog.base.set_state(169);
				let tmp = recog.enumSymbol()?;
				 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).enumSymbol = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,EnumDeclarationContext >(&mut _localctx).enumSymbol.clone().unwrap()
				 ;
				 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).enumSymbols.push(temp);
				  
				recog.base.set_state(174);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==Idl_Comma {
					{
					{
					recog.base.set_state(170);
					recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

					/*InvokeRule enumSymbol*/
					recog.base.set_state(171);
					let tmp = recog.enumSymbol()?;
					 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).enumSymbol = Some(tmp.clone());
					  

					let temp =  cast_mut::<_,EnumDeclarationContext >(&mut _localctx).enumSymbol.clone().unwrap()
					 ;
					 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).enumSymbols.push(temp);
					  
					}
					}
					recog.base.set_state(176);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(179);
			recog.base.match_token(Idl_RBrace,&mut recog.err_handler)?;

			recog.base.set_state(181);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_Equals {
				{
				/*InvokeRule enumDefault*/
				recog.base.set_state(180);
				let tmp = recog.enumDefault()?;
				 cast_mut::<_,EnumDeclarationContext >(&mut _localctx).defaultSymbol = Some(tmp.clone());
				  

				}
			}

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- enumSymbol ----------------
pub type EnumSymbolContextAll<'input> = EnumSymbolContext<'input>;


pub type EnumSymbolContext<'input> = BaseParserRuleContext<'input,EnumSymbolContextExt<'input>>;

#[derive(Clone)]
pub struct EnumSymbolContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for EnumSymbolContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for EnumSymbolContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_enumSymbol(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_enumSymbol(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for EnumSymbolContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_enumSymbol }
	//fn type_rule_index() -> usize where Self: Sized { RULE_enumSymbol }
}
antlr4rust::tid!{EnumSymbolContextExt<'a>}

impl<'input> EnumSymbolContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EnumSymbolContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EnumSymbolContextExt{
				doc: None, 
				schemaProperty: None, name: None, 
				schemaProperties: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait EnumSymbolContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<EnumSymbolContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> EnumSymbolContextAttrs<'input> for EnumSymbolContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn enumSymbol(&mut self,)
	-> Result<Rc<EnumSymbolContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EnumSymbolContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 22, RULE_enumSymbol);
        let mut _localctx: Rc<EnumSymbolContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(184);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(183);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,EnumSymbolContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(189);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(186);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,EnumSymbolContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,EnumSymbolContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,EnumSymbolContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(191);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule identifier*/
			recog.base.set_state(192);
			let tmp = recog.identifier()?;
			 cast_mut::<_,EnumSymbolContext >(&mut _localctx).name = Some(tmp.clone());
			  

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- enumDefault ----------------
pub type EnumDefaultContextAll<'input> = EnumDefaultContext<'input>;


pub type EnumDefaultContext<'input> = BaseParserRuleContext<'input,EnumDefaultContextExt<'input>>;

#[derive(Clone)]
pub struct EnumDefaultContextExt<'input>{
	pub defaultSymbolName: Option<Rc<IdentifierContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for EnumDefaultContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for EnumDefaultContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_enumDefault(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_enumDefault(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for EnumDefaultContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_enumDefault }
	//fn type_rule_index() -> usize where Self: Sized { RULE_enumDefault }
}
antlr4rust::tid!{EnumDefaultContextExt<'a>}

impl<'input> EnumDefaultContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EnumDefaultContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EnumDefaultContextExt{
				defaultSymbolName: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait EnumDefaultContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<EnumDefaultContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Equals
/// Returns `None` if there is no child corresponding to token Equals
fn Equals(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Equals, 0)
}
/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> EnumDefaultContextAttrs<'input> for EnumDefaultContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn enumDefault(&mut self,)
	-> Result<Rc<EnumDefaultContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EnumDefaultContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 24, RULE_enumDefault);
        let mut _localctx: Rc<EnumDefaultContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(194);
			recog.base.match_token(Idl_Equals,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(195);
			let tmp = recog.identifier()?;
			 cast_mut::<_,EnumDefaultContext >(&mut _localctx).defaultSymbolName = Some(tmp.clone());
			  

			recog.base.set_state(196);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- recordDeclaration ----------------
pub type RecordDeclarationContextAll<'input> = RecordDeclarationContext<'input>;


pub type RecordDeclarationContext<'input> = BaseParserRuleContext<'input,RecordDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct RecordDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub recordType: Option<TokenType<'input>>,
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
	pub body: Option<Rc<RecordBodyContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for RecordDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for RecordDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_recordDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_recordDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for RecordDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_recordDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_recordDeclaration }
}
antlr4rust::tid!{RecordDeclarationContextExt<'a>}

impl<'input> RecordDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<RecordDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,RecordDeclarationContextExt{
				doc: None, recordType: None, 
				schemaProperty: None, name: None, body: None, 
				schemaProperties: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait RecordDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<RecordDeclarationContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn recordBody(&self) -> Option<Rc<RecordBodyContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token Record
/// Returns `None` if there is no child corresponding to token Record
fn Record(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Record, 0)
}
/// Retrieves first TerminalNode corresponding to token Error
/// Returns `None` if there is no child corresponding to token Error
fn Error(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Error, 0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> RecordDeclarationContextAttrs<'input> for RecordDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn recordDeclaration(&mut self,)
	-> Result<Rc<RecordDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = RecordDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 26, RULE_recordDeclaration);
        let mut _localctx: Rc<RecordDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(199);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(198);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(204);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(201);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,RecordDeclarationContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(206);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(207);
			 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).recordType = recog.base.input.lt(1).cloned();
			 
			_la = recog.base.input.la(1);
			if { !(_la==Idl_Error || _la==Idl_Record) } {
				let tmp = recog.err_handler.recover_inline(&mut recog.base)?;
				 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).recordType = Some(tmp.clone());
				  

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			/*InvokeRule identifier*/
			recog.base.set_state(208);
			let tmp = recog.identifier()?;
			 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).name = Some(tmp.clone());
			  

			/*InvokeRule recordBody*/
			recog.base.set_state(209);
			let tmp = recog.recordBody()?;
			 cast_mut::<_,RecordDeclarationContext >(&mut _localctx).body = Some(tmp.clone());
			  

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- recordBody ----------------
pub type RecordBodyContextAll<'input> = RecordBodyContext<'input>;


pub type RecordBodyContext<'input> = BaseParserRuleContext<'input,RecordBodyContextExt<'input>>;

#[derive(Clone)]
pub struct RecordBodyContextExt<'input>{
	pub fieldDeclaration: Option<Rc<FieldDeclarationContextAll<'input>>>,
	pub fields:Vec<Rc<FieldDeclarationContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for RecordBodyContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for RecordBodyContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_recordBody(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_recordBody(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for RecordBodyContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_recordBody }
	//fn type_rule_index() -> usize where Self: Sized { RULE_recordBody }
}
antlr4rust::tid!{RecordBodyContextExt<'a>}

impl<'input> RecordBodyContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<RecordBodyContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,RecordBodyContextExt{
				fieldDeclaration: None, 
				fields: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait RecordBodyContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<RecordBodyContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token LBrace
/// Returns `None` if there is no child corresponding to token LBrace
fn LBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LBrace, 0)
}
/// Retrieves first TerminalNode corresponding to token RBrace
/// Returns `None` if there is no child corresponding to token RBrace
fn RBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RBrace, 0)
}
fn fieldDeclaration_all(&self) ->  Vec<Rc<FieldDeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn fieldDeclaration(&self, i: usize) -> Option<Rc<FieldDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> RecordBodyContextAttrs<'input> for RecordBodyContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn recordBody(&mut self,)
	-> Result<Rc<RecordBodyContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = RecordBodyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 28, RULE_recordBody);
        let mut _localctx: Rc<RecordBodyContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(211);
			recog.base.match_token(Idl_LBrace,&mut recog.err_handler)?;

			recog.base.set_state(215);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 4294967172) != 0) || ((((_la - 32)) & !0x3f) == 0 && ((1usize << (_la - 32)) & 33587263) != 0) {
				{
				{
				/*InvokeRule fieldDeclaration*/
				recog.base.set_state(212);
				let tmp = recog.fieldDeclaration()?;
				 cast_mut::<_,RecordBodyContext >(&mut _localctx).fieldDeclaration = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,RecordBodyContext >(&mut _localctx).fieldDeclaration.clone().unwrap()
				 ;
				 cast_mut::<_,RecordBodyContext >(&mut _localctx).fields.push(temp);
				  
				}
				}
				recog.base.set_state(217);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(218);
			recog.base.match_token(Idl_RBrace,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- fieldDeclaration ----------------
pub type FieldDeclarationContextAll<'input> = FieldDeclarationContext<'input>;


pub type FieldDeclarationContext<'input> = BaseParserRuleContext<'input,FieldDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct FieldDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub fieldType: Option<Rc<FullTypeContextAll<'input>>>,
	pub variableDeclaration: Option<Rc<VariableDeclarationContextAll<'input>>>,
	pub variableDeclarations:Vec<Rc<VariableDeclarationContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for FieldDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for FieldDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_fieldDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_fieldDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for FieldDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_fieldDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_fieldDeclaration }
}
antlr4rust::tid!{FieldDeclarationContextExt<'a>}

impl<'input> FieldDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<FieldDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,FieldDeclarationContextExt{
				doc: None, 
				fieldType: None, variableDeclaration: None, 
				variableDeclarations: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait FieldDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<FieldDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
fn fullType(&self) -> Option<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn variableDeclaration_all(&self) ->  Vec<Rc<VariableDeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn variableDeclaration(&self, i: usize) -> Option<Rc<VariableDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token Comma in current rule
fn Comma_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token Comma, starting from 0.
/// Returns `None` if number of children corresponding to token Comma is less or equal than `i`.
fn Comma(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, i)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}

}

impl<'input> FieldDeclarationContextAttrs<'input> for FieldDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn fieldDeclaration(&mut self,)
	-> Result<Rc<FieldDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = FieldDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 30, RULE_fieldDeclaration);
        let mut _localctx: Rc<FieldDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(221);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(220);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,FieldDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			/*InvokeRule fullType*/
			recog.base.set_state(223);
			let tmp = recog.fullType()?;
			 cast_mut::<_,FieldDeclarationContext >(&mut _localctx).fieldType = Some(tmp.clone());
			  

			/*InvokeRule variableDeclaration*/
			recog.base.set_state(224);
			let tmp = recog.variableDeclaration()?;
			 cast_mut::<_,FieldDeclarationContext >(&mut _localctx).variableDeclaration = Some(tmp.clone());
			  

			let temp =  cast_mut::<_,FieldDeclarationContext >(&mut _localctx).variableDeclaration.clone().unwrap()
			 ;
			 cast_mut::<_,FieldDeclarationContext >(&mut _localctx).variableDeclarations.push(temp);
			  
			recog.base.set_state(229);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_Comma {
				{
				{
				recog.base.set_state(225);
				recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

				/*InvokeRule variableDeclaration*/
				recog.base.set_state(226);
				let tmp = recog.variableDeclaration()?;
				 cast_mut::<_,FieldDeclarationContext >(&mut _localctx).variableDeclaration = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,FieldDeclarationContext >(&mut _localctx).variableDeclaration.clone().unwrap()
				 ;
				 cast_mut::<_,FieldDeclarationContext >(&mut _localctx).variableDeclarations.push(temp);
				  
				}
				}
				recog.base.set_state(231);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(232);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- variableDeclaration ----------------
pub type VariableDeclarationContextAll<'input> = VariableDeclarationContext<'input>;


pub type VariableDeclarationContext<'input> = BaseParserRuleContext<'input,VariableDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct VariableDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub fieldName: Option<Rc<IdentifierContextAll<'input>>>,
	pub defaultValue: Option<Rc<JsonValueContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for VariableDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for VariableDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_variableDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_variableDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for VariableDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_variableDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_variableDeclaration }
}
antlr4rust::tid!{VariableDeclarationContextExt<'a>}

impl<'input> VariableDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<VariableDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,VariableDeclarationContextExt{
				doc: None, 
				schemaProperty: None, fieldName: None, defaultValue: None, 
				schemaProperties: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait VariableDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<VariableDeclarationContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token Equals
/// Returns `None` if there is no child corresponding to token Equals
fn Equals(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Equals, 0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> VariableDeclarationContextAttrs<'input> for VariableDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn variableDeclaration(&mut self,)
	-> Result<Rc<VariableDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = VariableDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 32, RULE_variableDeclaration);
        let mut _localctx: Rc<VariableDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(235);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(234);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,VariableDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(240);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(237);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,VariableDeclarationContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,VariableDeclarationContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,VariableDeclarationContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(242);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule identifier*/
			recog.base.set_state(243);
			let tmp = recog.identifier()?;
			 cast_mut::<_,VariableDeclarationContext >(&mut _localctx).fieldName = Some(tmp.clone());
			  

			recog.base.set_state(246);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_Equals {
				{
				recog.base.set_state(244);
				recog.base.match_token(Idl_Equals,&mut recog.err_handler)?;

				/*InvokeRule jsonValue*/
				recog.base.set_state(245);
				let tmp = recog.jsonValue()?;
				 cast_mut::<_,VariableDeclarationContext >(&mut _localctx).defaultValue = Some(tmp.clone());
				  

				}
			}

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- messageDeclaration ----------------
pub type MessageDeclarationContextAll<'input> = MessageDeclarationContext<'input>;


pub type MessageDeclarationContext<'input> = BaseParserRuleContext<'input,MessageDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct MessageDeclarationContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
	pub returnType: Option<Rc<ResultTypeContextAll<'input>>>,
	pub name: Option<Rc<IdentifierContextAll<'input>>>,
	pub formalParameter: Option<Rc<FormalParameterContextAll<'input>>>,
	pub formalParameters:Vec<Rc<FormalParameterContextAll<'input>>>,
	pub oneway: Option<TokenType<'input>>,
	pub identifier: Option<Rc<IdentifierContextAll<'input>>>,
	pub errors:Vec<Rc<IdentifierContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for MessageDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for MessageDeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_messageDeclaration(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_messageDeclaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for MessageDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_messageDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_messageDeclaration }
}
antlr4rust::tid!{MessageDeclarationContextExt<'a>}

impl<'input> MessageDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<MessageDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MessageDeclarationContextExt{
				doc: None, oneway: None, 
				schemaProperty: None, returnType: None, name: None, formalParameter: None, identifier: None, 
				schemaProperties: Vec::new(), formalParameters: Vec::new(), errors: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait MessageDeclarationContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<MessageDeclarationContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token LParen
/// Returns `None` if there is no child corresponding to token LParen
fn LParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LParen, 0)
}
/// Retrieves first TerminalNode corresponding to token RParen
/// Returns `None` if there is no child corresponding to token RParen
fn RParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RParen, 0)
}
/// Retrieves first TerminalNode corresponding to token Semicolon
/// Returns `None` if there is no child corresponding to token Semicolon
fn Semicolon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Semicolon, 0)
}
fn resultType(&self) -> Option<Rc<ResultTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn identifier_all(&self) ->  Vec<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn identifier(&self, i: usize) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves first TerminalNode corresponding to token Throws
/// Returns `None` if there is no child corresponding to token Throws
fn Throws(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Throws, 0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn formalParameter_all(&self) ->  Vec<Rc<FormalParameterContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn formalParameter(&self, i: usize) -> Option<Rc<FormalParameterContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves first TerminalNode corresponding to token Oneway
/// Returns `None` if there is no child corresponding to token Oneway
fn Oneway(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Oneway, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token Comma in current rule
fn Comma_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token Comma, starting from 0.
/// Returns `None` if number of children corresponding to token Comma is less or equal than `i`.
fn Comma(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, i)
}

}

impl<'input> MessageDeclarationContextAttrs<'input> for MessageDeclarationContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn messageDeclaration(&mut self,)
	-> Result<Rc<MessageDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MessageDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 34, RULE_messageDeclaration);
        let mut _localctx: Rc<MessageDeclarationContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(249);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(248);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			recog.base.set_state(254);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(251);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,MessageDeclarationContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(256);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule resultType*/
			recog.base.set_state(257);
			let tmp = recog.resultType()?;
			 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).returnType = Some(tmp.clone());
			  

			/*InvokeRule identifier*/
			recog.base.set_state(258);
			let tmp = recog.identifier()?;
			 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).name = Some(tmp.clone());
			  

			recog.base.set_state(259);
			recog.base.match_token(Idl_LParen,&mut recog.err_handler)?;

			recog.base.set_state(268);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 4294967172) != 0) || ((((_la - 32)) & !0x3f) == 0 && ((1usize << (_la - 32)) & 33587263) != 0) {
				{
				/*InvokeRule formalParameter*/
				recog.base.set_state(260);
				let tmp = recog.formalParameter()?;
				 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).formalParameter = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,MessageDeclarationContext >(&mut _localctx).formalParameter.clone().unwrap()
				 ;
				 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).formalParameters.push(temp);
				  
				recog.base.set_state(265);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==Idl_Comma {
					{
					{
					recog.base.set_state(261);
					recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

					/*InvokeRule formalParameter*/
					recog.base.set_state(262);
					let tmp = recog.formalParameter()?;
					 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).formalParameter = Some(tmp.clone());
					  

					let temp =  cast_mut::<_,MessageDeclarationContext >(&mut _localctx).formalParameter.clone().unwrap()
					 ;
					 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).formalParameters.push(temp);
					  
					}
					}
					recog.base.set_state(267);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(270);
			recog.base.match_token(Idl_RParen,&mut recog.err_handler)?;

			recog.base.set_state(281);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Idl_Oneway 
				=> {
			    	{
			    	recog.base.set_state(271);
			    	let tmp = recog.base.match_token(Idl_Oneway,&mut recog.err_handler)?;
			    	 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).oneway = Some(tmp.clone());
			    	  

			    	}
			    }

			Idl_Throws 
				=> {
			    	{
			    	recog.base.set_state(272);
			    	recog.base.match_token(Idl_Throws,&mut recog.err_handler)?;

			    	/*InvokeRule identifier*/
			    	recog.base.set_state(273);
			    	let tmp = recog.identifier()?;
			    	 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).identifier = Some(tmp.clone());
			    	  

			    	let temp =  cast_mut::<_,MessageDeclarationContext >(&mut _localctx).identifier.clone().unwrap()
			    	 ;
			    	 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).errors.push(temp);
			    	  
			    	recog.base.set_state(278);
			    	recog.err_handler.sync(&mut recog.base)?;
			    	_la = recog.base.input.la(1);
			    	while _la==Idl_Comma {
			    		{
			    		{
			    		recog.base.set_state(274);
			    		recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

			    		/*InvokeRule identifier*/
			    		recog.base.set_state(275);
			    		let tmp = recog.identifier()?;
			    		 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).identifier = Some(tmp.clone());
			    		  

			    		let temp =  cast_mut::<_,MessageDeclarationContext >(&mut _localctx).identifier.clone().unwrap()
			    		 ;
			    		 cast_mut::<_,MessageDeclarationContext >(&mut _localctx).errors.push(temp);
			    		  
			    		}
			    		}
			    		recog.base.set_state(280);
			    		recog.err_handler.sync(&mut recog.base)?;
			    		_la = recog.base.input.la(1);
			    	}
			    	}
			    }

			Idl_Semicolon 
				=> {
			    }

				_ => {}
			}
			recog.base.set_state(283);
			recog.base.match_token(Idl_Semicolon,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- formalParameter ----------------
pub type FormalParameterContextAll<'input> = FormalParameterContext<'input>;


pub type FormalParameterContext<'input> = BaseParserRuleContext<'input,FormalParameterContextExt<'input>>;

#[derive(Clone)]
pub struct FormalParameterContextExt<'input>{
	pub doc: Option<TokenType<'input>>,
	pub parameterType: Option<Rc<FullTypeContextAll<'input>>>,
	pub parameter: Option<Rc<VariableDeclarationContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for FormalParameterContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for FormalParameterContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_formalParameter(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_formalParameter(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for FormalParameterContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_formalParameter }
	//fn type_rule_index() -> usize where Self: Sized { RULE_formalParameter }
}
antlr4rust::tid!{FormalParameterContextExt<'a>}

impl<'input> FormalParameterContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<FormalParameterContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,FormalParameterContextExt{
				doc: None, 
				parameterType: None, parameter: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait FormalParameterContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<FormalParameterContextExt<'input>>{

fn fullType(&self) -> Option<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn variableDeclaration(&self) -> Option<Rc<VariableDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token DocComment
/// Returns `None` if there is no child corresponding to token DocComment
fn DocComment(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_DocComment, 0)
}

}

impl<'input> FormalParameterContextAttrs<'input> for FormalParameterContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn formalParameter(&mut self,)
	-> Result<Rc<FormalParameterContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = FormalParameterContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 36, RULE_formalParameter);
        let mut _localctx: Rc<FormalParameterContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(286);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_DocComment {
				{
				recog.base.set_state(285);
				let tmp = recog.base.match_token(Idl_DocComment,&mut recog.err_handler)?;
				 cast_mut::<_,FormalParameterContext >(&mut _localctx).doc = Some(tmp.clone());
				  

				}
			}

			/*InvokeRule fullType*/
			recog.base.set_state(288);
			let tmp = recog.fullType()?;
			 cast_mut::<_,FormalParameterContext >(&mut _localctx).parameterType = Some(tmp.clone());
			  

			/*InvokeRule variableDeclaration*/
			recog.base.set_state(289);
			let tmp = recog.variableDeclaration()?;
			 cast_mut::<_,FormalParameterContext >(&mut _localctx).parameter = Some(tmp.clone());
			  

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- resultType ----------------
pub type ResultTypeContextAll<'input> = ResultTypeContext<'input>;


pub type ResultTypeContext<'input> = BaseParserRuleContext<'input,ResultTypeContextExt<'input>>;

#[derive(Clone)]
pub struct ResultTypeContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for ResultTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for ResultTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_resultType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_resultType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for ResultTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_resultType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_resultType }
}
antlr4rust::tid!{ResultTypeContextExt<'a>}

impl<'input> ResultTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ResultTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ResultTypeContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ResultTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<ResultTypeContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Void
/// Returns `None` if there is no child corresponding to token Void
fn Void(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Void, 0)
}
fn plainType(&self) -> Option<Rc<PlainTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ResultTypeContextAttrs<'input> for ResultTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn resultType(&mut self,)
	-> Result<Rc<ResultTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ResultTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 38, RULE_resultType);
        let mut _localctx: Rc<ResultTypeContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(293);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(36,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(291);
					recog.base.match_token(Idl_Void,&mut recog.err_handler)?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule plainType*/
					recog.base.set_state(292);
					recog.plainType()?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- fullType ----------------
pub type FullTypeContextAll<'input> = FullTypeContext<'input>;


pub type FullTypeContext<'input> = BaseParserRuleContext<'input,FullTypeContextExt<'input>>;

#[derive(Clone)]
pub struct FullTypeContextExt<'input>{
	pub schemaProperty: Option<Rc<SchemaPropertyContextAll<'input>>>,
	pub schemaProperties:Vec<Rc<SchemaPropertyContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for FullTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for FullTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_fullType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_fullType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for FullTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_fullType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_fullType }
}
antlr4rust::tid!{FullTypeContextExt<'a>}

impl<'input> FullTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<FullTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,FullTypeContextExt{
				schemaProperty: None, 
				schemaProperties: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait FullTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<FullTypeContextExt<'input>>{

fn plainType(&self) -> Option<Rc<PlainTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn schemaProperty_all(&self) ->  Vec<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn schemaProperty(&self, i: usize) -> Option<Rc<SchemaPropertyContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> FullTypeContextAttrs<'input> for FullTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn fullType(&mut self,)
	-> Result<Rc<FullTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = FullTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 40, RULE_fullType);
        let mut _localctx: Rc<FullTypeContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(298);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_At {
				{
				{
				/*InvokeRule schemaProperty*/
				recog.base.set_state(295);
				let tmp = recog.schemaProperty()?;
				 cast_mut::<_,FullTypeContext >(&mut _localctx).schemaProperty = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,FullTypeContext >(&mut _localctx).schemaProperty.clone().unwrap()
				 ;
				 cast_mut::<_,FullTypeContext >(&mut _localctx).schemaProperties.push(temp);
				  
				}
				}
				recog.base.set_state(300);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule plainType*/
			recog.base.set_state(301);
			recog.plainType()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- plainType ----------------
pub type PlainTypeContextAll<'input> = PlainTypeContext<'input>;


pub type PlainTypeContext<'input> = BaseParserRuleContext<'input,PlainTypeContextExt<'input>>;

#[derive(Clone)]
pub struct PlainTypeContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for PlainTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for PlainTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_plainType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_plainType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for PlainTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_plainType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_plainType }
}
antlr4rust::tid!{PlainTypeContextExt<'a>}

impl<'input> PlainTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PlainTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PlainTypeContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PlainTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<PlainTypeContextExt<'input>>{

fn arrayType(&self) -> Option<Rc<ArrayTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn mapType(&self) -> Option<Rc<MapTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn unionType(&self) -> Option<Rc<UnionTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn nullableType(&self) -> Option<Rc<NullableTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PlainTypeContextAttrs<'input> for PlainTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn plainType(&mut self,)
	-> Result<Rc<PlainTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PlainTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 42, RULE_plainType);
        let mut _localctx: Rc<PlainTypeContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(307);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(38,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule arrayType*/
					recog.base.set_state(303);
					recog.arrayType()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule mapType*/
					recog.base.set_state(304);
					recog.mapType()?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule unionType*/
					recog.base.set_state(305);
					recog.unionType()?;

					}
				}
			,
				4 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule nullableType*/
					recog.base.set_state(306);
					recog.nullableType()?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- nullableType ----------------
pub type NullableTypeContextAll<'input> = NullableTypeContext<'input>;


pub type NullableTypeContext<'input> = BaseParserRuleContext<'input,NullableTypeContextExt<'input>>;

#[derive(Clone)]
pub struct NullableTypeContextExt<'input>{
	pub referenceName: Option<Rc<IdentifierContextAll<'input>>>,
	pub optional: Option<TokenType<'input>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for NullableTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for NullableTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_nullableType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_nullableType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for NullableTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_nullableType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_nullableType }
}
antlr4rust::tid!{NullableTypeContextExt<'a>}

impl<'input> NullableTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<NullableTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,NullableTypeContextExt{
				optional: None, 
				referenceName: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait NullableTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<NullableTypeContextExt<'input>>{

fn primitiveType(&self) -> Option<Rc<PrimitiveTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token QuestionMark
/// Returns `None` if there is no child corresponding to token QuestionMark
fn QuestionMark(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_QuestionMark, 0)
}

}

impl<'input> NullableTypeContextAttrs<'input> for NullableTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn nullableType(&mut self,)
	-> Result<Rc<NullableTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = NullableTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 44, RULE_nullableType);
        let mut _localctx: Rc<NullableTypeContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(311);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(39,&mut recog.base)? {
				1 =>{
					{
					/*InvokeRule primitiveType*/
					recog.base.set_state(309);
					recog.primitiveType()?;

					}
				}
			,
				2 =>{
					{
					/*InvokeRule identifier*/
					recog.base.set_state(310);
					let tmp = recog.identifier()?;
					 cast_mut::<_,NullableTypeContext >(&mut _localctx).referenceName = Some(tmp.clone());
					  

					}
				}

				_ => {}
			}
			recog.base.set_state(314);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_QuestionMark {
				{
				recog.base.set_state(313);
				let tmp = recog.base.match_token(Idl_QuestionMark,&mut recog.err_handler)?;
				 cast_mut::<_,NullableTypeContext >(&mut _localctx).optional = Some(tmp.clone());
				  

				}
			}

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- primitiveType ----------------
pub type PrimitiveTypeContextAll<'input> = PrimitiveTypeContext<'input>;


pub type PrimitiveTypeContext<'input> = BaseParserRuleContext<'input,PrimitiveTypeContextExt<'input>>;

#[derive(Clone)]
pub struct PrimitiveTypeContextExt<'input>{
	pub typeName: Option<TokenType<'input>>,
	pub precision: Option<TokenType<'input>>,
	pub scale: Option<TokenType<'input>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for PrimitiveTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for PrimitiveTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_primitiveType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_primitiveType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for PrimitiveTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_primitiveType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_primitiveType }
}
antlr4rust::tid!{PrimitiveTypeContextExt<'a>}

impl<'input> PrimitiveTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PrimitiveTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PrimitiveTypeContextExt{
				typeName: None, precision: None, scale: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait PrimitiveTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<PrimitiveTypeContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Boolean
/// Returns `None` if there is no child corresponding to token Boolean
fn Boolean(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Boolean, 0)
}
/// Retrieves first TerminalNode corresponding to token Int
/// Returns `None` if there is no child corresponding to token Int
fn Int(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Int, 0)
}
/// Retrieves first TerminalNode corresponding to token Long
/// Returns `None` if there is no child corresponding to token Long
fn Long(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Long, 0)
}
/// Retrieves first TerminalNode corresponding to token Float
/// Returns `None` if there is no child corresponding to token Float
fn Float(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Float, 0)
}
/// Retrieves first TerminalNode corresponding to token Double
/// Returns `None` if there is no child corresponding to token Double
fn Double(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Double, 0)
}
/// Retrieves first TerminalNode corresponding to token Bytes
/// Returns `None` if there is no child corresponding to token Bytes
fn Bytes(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Bytes, 0)
}
/// Retrieves first TerminalNode corresponding to token String
/// Returns `None` if there is no child corresponding to token String
fn String(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_String, 0)
}
/// Retrieves first TerminalNode corresponding to token Null
/// Returns `None` if there is no child corresponding to token Null
fn Null(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Null, 0)
}
/// Retrieves first TerminalNode corresponding to token Date
/// Returns `None` if there is no child corresponding to token Date
fn Date(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Date, 0)
}
/// Retrieves first TerminalNode corresponding to token Time
/// Returns `None` if there is no child corresponding to token Time
fn Time(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Time, 0)
}
/// Retrieves first TerminalNode corresponding to token Timestamp
/// Returns `None` if there is no child corresponding to token Timestamp
fn Timestamp(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Timestamp, 0)
}
/// Retrieves first TerminalNode corresponding to token LocalTimestamp
/// Returns `None` if there is no child corresponding to token LocalTimestamp
fn LocalTimestamp(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LocalTimestamp, 0)
}
/// Retrieves first TerminalNode corresponding to token UUID
/// Returns `None` if there is no child corresponding to token UUID
fn UUID(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_UUID, 0)
}
/// Retrieves first TerminalNode corresponding to token LParen
/// Returns `None` if there is no child corresponding to token LParen
fn LParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LParen, 0)
}
/// Retrieves first TerminalNode corresponding to token RParen
/// Returns `None` if there is no child corresponding to token RParen
fn RParen(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RParen, 0)
}
/// Retrieves first TerminalNode corresponding to token Decimal
/// Returns `None` if there is no child corresponding to token Decimal
fn Decimal(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Decimal, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token IntegerLiteral in current rule
fn IntegerLiteral_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token IntegerLiteral, starting from 0.
/// Returns `None` if number of children corresponding to token IntegerLiteral is less or equal than `i`.
fn IntegerLiteral(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_IntegerLiteral, i)
}
/// Retrieves first TerminalNode corresponding to token Comma
/// Returns `None` if there is no child corresponding to token Comma
fn Comma(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, 0)
}

}

impl<'input> PrimitiveTypeContextAttrs<'input> for PrimitiveTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn primitiveType(&mut self,)
	-> Result<Rc<PrimitiveTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PrimitiveTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 46, RULE_primitiveType);
        let mut _localctx: Rc<PrimitiveTypeContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(325);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Idl_Boolean |Idl_Int |Idl_Long |Idl_Float |Idl_Double |Idl_String |Idl_Bytes |
			Idl_Null |Idl_Date |Idl_Time |Idl_Timestamp |Idl_LocalTimestamp |Idl_UUID 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(316);
					 cast_mut::<_,PrimitiveTypeContext >(&mut _localctx).typeName = recog.base.input.lt(1).cloned();
					 
					_la = recog.base.input.la(1);
					if { !(((((_la - 19)) & !0x3f) == 0 && ((1usize << (_la - 19)) & 63743) != 0)) } {
						let tmp = recog.err_handler.recover_inline(&mut recog.base)?;
						 cast_mut::<_,PrimitiveTypeContext >(&mut _localctx).typeName = Some(tmp.clone());
						  

					}
					else {
						if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
						recog.err_handler.report_match(&mut recog.base);
						recog.base.consume(&mut recog.err_handler);
					}
					}
				}

			Idl_Decimal 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(317);
					let tmp = recog.base.match_token(Idl_Decimal,&mut recog.err_handler)?;
					 cast_mut::<_,PrimitiveTypeContext >(&mut _localctx).typeName = Some(tmp.clone());
					  

					recog.base.set_state(318);
					recog.base.match_token(Idl_LParen,&mut recog.err_handler)?;

					recog.base.set_state(319);
					let tmp = recog.base.match_token(Idl_IntegerLiteral,&mut recog.err_handler)?;
					 cast_mut::<_,PrimitiveTypeContext >(&mut _localctx).precision = Some(tmp.clone());
					  

					recog.base.set_state(322);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if _la==Idl_Comma {
						{
						recog.base.set_state(320);
						recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

						recog.base.set_state(321);
						let tmp = recog.base.match_token(Idl_IntegerLiteral,&mut recog.err_handler)?;
						 cast_mut::<_,PrimitiveTypeContext >(&mut _localctx).scale = Some(tmp.clone());
						  

						}
					}

					recog.base.set_state(324);
					recog.base.match_token(Idl_RParen,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- arrayType ----------------
pub type ArrayTypeContextAll<'input> = ArrayTypeContext<'input>;


pub type ArrayTypeContext<'input> = BaseParserRuleContext<'input,ArrayTypeContextExt<'input>>;

#[derive(Clone)]
pub struct ArrayTypeContextExt<'input>{
	pub elementType: Option<Rc<FullTypeContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for ArrayTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for ArrayTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_arrayType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_arrayType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for ArrayTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_arrayType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_arrayType }
}
antlr4rust::tid!{ArrayTypeContextExt<'a>}

impl<'input> ArrayTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ArrayTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ArrayTypeContextExt{
				elementType: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait ArrayTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<ArrayTypeContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Array
/// Returns `None` if there is no child corresponding to token Array
fn Array(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Array, 0)
}
/// Retrieves first TerminalNode corresponding to token LT
/// Returns `None` if there is no child corresponding to token LT
fn LT(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LT, 0)
}
/// Retrieves first TerminalNode corresponding to token GT
/// Returns `None` if there is no child corresponding to token GT
fn GT(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_GT, 0)
}
fn fullType(&self) -> Option<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ArrayTypeContextAttrs<'input> for ArrayTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn arrayType(&mut self,)
	-> Result<Rc<ArrayTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ArrayTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 48, RULE_arrayType);
        let mut _localctx: Rc<ArrayTypeContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(327);
			recog.base.match_token(Idl_Array,&mut recog.err_handler)?;

			recog.base.set_state(328);
			recog.base.match_token(Idl_LT,&mut recog.err_handler)?;

			/*InvokeRule fullType*/
			recog.base.set_state(329);
			let tmp = recog.fullType()?;
			 cast_mut::<_,ArrayTypeContext >(&mut _localctx).elementType = Some(tmp.clone());
			  

			recog.base.set_state(330);
			recog.base.match_token(Idl_GT,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- mapType ----------------
pub type MapTypeContextAll<'input> = MapTypeContext<'input>;


pub type MapTypeContext<'input> = BaseParserRuleContext<'input,MapTypeContextExt<'input>>;

#[derive(Clone)]
pub struct MapTypeContextExt<'input>{
	pub valueType: Option<Rc<FullTypeContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for MapTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for MapTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_mapType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_mapType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for MapTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_mapType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_mapType }
}
antlr4rust::tid!{MapTypeContextExt<'a>}

impl<'input> MapTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<MapTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MapTypeContextExt{
				valueType: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait MapTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<MapTypeContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Map
/// Returns `None` if there is no child corresponding to token Map
fn Map(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Map, 0)
}
/// Retrieves first TerminalNode corresponding to token LT
/// Returns `None` if there is no child corresponding to token LT
fn LT(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LT, 0)
}
/// Retrieves first TerminalNode corresponding to token GT
/// Returns `None` if there is no child corresponding to token GT
fn GT(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_GT, 0)
}
fn fullType(&self) -> Option<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> MapTypeContextAttrs<'input> for MapTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn mapType(&mut self,)
	-> Result<Rc<MapTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MapTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 50, RULE_mapType);
        let mut _localctx: Rc<MapTypeContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(332);
			recog.base.match_token(Idl_Map,&mut recog.err_handler)?;

			recog.base.set_state(333);
			recog.base.match_token(Idl_LT,&mut recog.err_handler)?;

			/*InvokeRule fullType*/
			recog.base.set_state(334);
			let tmp = recog.fullType()?;
			 cast_mut::<_,MapTypeContext >(&mut _localctx).valueType = Some(tmp.clone());
			  

			recog.base.set_state(335);
			recog.base.match_token(Idl_GT,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- unionType ----------------
pub type UnionTypeContextAll<'input> = UnionTypeContext<'input>;


pub type UnionTypeContext<'input> = BaseParserRuleContext<'input,UnionTypeContextExt<'input>>;

#[derive(Clone)]
pub struct UnionTypeContextExt<'input>{
	pub fullType: Option<Rc<FullTypeContextAll<'input>>>,
	pub types:Vec<Rc<FullTypeContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for UnionTypeContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for UnionTypeContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_unionType(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_unionType(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for UnionTypeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_unionType }
	//fn type_rule_index() -> usize where Self: Sized { RULE_unionType }
}
antlr4rust::tid!{UnionTypeContextExt<'a>}

impl<'input> UnionTypeContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<UnionTypeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,UnionTypeContextExt{
				fullType: None, 
				types: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait UnionTypeContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<UnionTypeContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Union
/// Returns `None` if there is no child corresponding to token Union
fn Union(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Union, 0)
}
/// Retrieves first TerminalNode corresponding to token LBrace
/// Returns `None` if there is no child corresponding to token LBrace
fn LBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LBrace, 0)
}
/// Retrieves first TerminalNode corresponding to token RBrace
/// Returns `None` if there is no child corresponding to token RBrace
fn RBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RBrace, 0)
}
fn fullType_all(&self) ->  Vec<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn fullType(&self, i: usize) -> Option<Rc<FullTypeContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token Comma in current rule
fn Comma_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token Comma, starting from 0.
/// Returns `None` if number of children corresponding to token Comma is less or equal than `i`.
fn Comma(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, i)
}

}

impl<'input> UnionTypeContextAttrs<'input> for UnionTypeContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn unionType(&mut self,)
	-> Result<Rc<UnionTypeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = UnionTypeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 52, RULE_unionType);
        let mut _localctx: Rc<UnionTypeContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(337);
			recog.base.match_token(Idl_Union,&mut recog.err_handler)?;

			recog.base.set_state(338);
			recog.base.match_token(Idl_LBrace,&mut recog.err_handler)?;

			/*InvokeRule fullType*/
			recog.base.set_state(339);
			let tmp = recog.fullType()?;
			 cast_mut::<_,UnionTypeContext >(&mut _localctx).fullType = Some(tmp.clone());
			  

			let temp =  cast_mut::<_,UnionTypeContext >(&mut _localctx).fullType.clone().unwrap()
			 ;
			 cast_mut::<_,UnionTypeContext >(&mut _localctx).types.push(temp);
			  
			recog.base.set_state(344);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Idl_Comma {
				{
				{
				recog.base.set_state(340);
				recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

				/*InvokeRule fullType*/
				recog.base.set_state(341);
				let tmp = recog.fullType()?;
				 cast_mut::<_,UnionTypeContext >(&mut _localctx).fullType = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,UnionTypeContext >(&mut _localctx).fullType.clone().unwrap()
				 ;
				 cast_mut::<_,UnionTypeContext >(&mut _localctx).types.push(temp);
				  
				}
				}
				recog.base.set_state(346);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(347);
			recog.base.match_token(Idl_RBrace,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonValue ----------------
pub type JsonValueContextAll<'input> = JsonValueContext<'input>;


pub type JsonValueContext<'input> = BaseParserRuleContext<'input,JsonValueContextExt<'input>>;

#[derive(Clone)]
pub struct JsonValueContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for JsonValueContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for JsonValueContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonValue(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonValue(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for JsonValueContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonValue }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonValue }
}
antlr4rust::tid!{JsonValueContextExt<'a>}

impl<'input> JsonValueContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonValueContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonValueContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonValueContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<JsonValueContextExt<'input>>{

fn jsonObject(&self) -> Option<Rc<JsonObjectContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn jsonArray(&self) -> Option<Rc<JsonArrayContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn jsonLiteral(&self) -> Option<Rc<JsonLiteralContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> JsonValueContextAttrs<'input> for JsonValueContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonValue(&mut self,)
	-> Result<Rc<JsonValueContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonValueContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 54, RULE_jsonValue);
        let mut _localctx: Rc<JsonValueContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(352);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Idl_LBrace 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule jsonObject*/
					recog.base.set_state(349);
					recog.jsonObject()?;

					}
				}

			Idl_LBracket 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule jsonArray*/
					recog.base.set_state(350);
					recog.jsonArray()?;

					}
				}

			Idl_Null |Idl_BTrue |Idl_BFalse |Idl_StringLiteral |Idl_IntegerLiteral |
			Idl_FloatingPointLiteral 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule jsonLiteral*/
					recog.base.set_state(351);
					recog.jsonLiteral()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonLiteral ----------------
pub type JsonLiteralContextAll<'input> = JsonLiteralContext<'input>;


pub type JsonLiteralContext<'input> = BaseParserRuleContext<'input,JsonLiteralContextExt<'input>>;

#[derive(Clone)]
pub struct JsonLiteralContextExt<'input>{
	pub literal: Option<TokenType<'input>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for JsonLiteralContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for JsonLiteralContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonLiteral(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonLiteral(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for JsonLiteralContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonLiteral }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonLiteral }
}
antlr4rust::tid!{JsonLiteralContextExt<'a>}

impl<'input> JsonLiteralContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonLiteralContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonLiteralContextExt{
				literal: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonLiteralContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<JsonLiteralContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token StringLiteral
/// Returns `None` if there is no child corresponding to token StringLiteral
fn StringLiteral(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_StringLiteral, 0)
}
/// Retrieves first TerminalNode corresponding to token IntegerLiteral
/// Returns `None` if there is no child corresponding to token IntegerLiteral
fn IntegerLiteral(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_IntegerLiteral, 0)
}
/// Retrieves first TerminalNode corresponding to token FloatingPointLiteral
/// Returns `None` if there is no child corresponding to token FloatingPointLiteral
fn FloatingPointLiteral(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_FloatingPointLiteral, 0)
}
/// Retrieves first TerminalNode corresponding to token BTrue
/// Returns `None` if there is no child corresponding to token BTrue
fn BTrue(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_BTrue, 0)
}
/// Retrieves first TerminalNode corresponding to token BFalse
/// Returns `None` if there is no child corresponding to token BFalse
fn BFalse(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_BFalse, 0)
}
/// Retrieves first TerminalNode corresponding to token Null
/// Returns `None` if there is no child corresponding to token Null
fn Null(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Null, 0)
}

}

impl<'input> JsonLiteralContextAttrs<'input> for JsonLiteralContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonLiteral(&mut self,)
	-> Result<Rc<JsonLiteralContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonLiteralContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 56, RULE_jsonLiteral);
        let mut _localctx: Rc<JsonLiteralContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(354);
			 cast_mut::<_,JsonLiteralContext >(&mut _localctx).literal = recog.base.input.lt(1).cloned();
			 
			_la = recog.base.input.la(1);
			if { !(((((_la - 26)) & !0x3f) == 0 && ((1usize << (_la - 26)) & 1879048199) != 0)) } {
				let tmp = recog.err_handler.recover_inline(&mut recog.base)?;
				 cast_mut::<_,JsonLiteralContext >(&mut _localctx).literal = Some(tmp.clone());
				  

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonObject ----------------
pub type JsonObjectContextAll<'input> = JsonObjectContext<'input>;


pub type JsonObjectContext<'input> = BaseParserRuleContext<'input,JsonObjectContextExt<'input>>;

#[derive(Clone)]
pub struct JsonObjectContextExt<'input>{
	pub jsonPair: Option<Rc<JsonPairContextAll<'input>>>,
	pub jsonPairs:Vec<Rc<JsonPairContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for JsonObjectContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for JsonObjectContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonObject(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonObject(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for JsonObjectContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonObject }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonObject }
}
antlr4rust::tid!{JsonObjectContextExt<'a>}

impl<'input> JsonObjectContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonObjectContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonObjectContextExt{
				jsonPair: None, 
				jsonPairs: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonObjectContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<JsonObjectContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token LBrace
/// Returns `None` if there is no child corresponding to token LBrace
fn LBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LBrace, 0)
}
/// Retrieves first TerminalNode corresponding to token RBrace
/// Returns `None` if there is no child corresponding to token RBrace
fn RBrace(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RBrace, 0)
}
fn jsonPair_all(&self) ->  Vec<Rc<JsonPairContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn jsonPair(&self, i: usize) -> Option<Rc<JsonPairContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token Comma in current rule
fn Comma_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token Comma, starting from 0.
/// Returns `None` if number of children corresponding to token Comma is less or equal than `i`.
fn Comma(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, i)
}

}

impl<'input> JsonObjectContextAttrs<'input> for JsonObjectContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonObject(&mut self,)
	-> Result<Rc<JsonObjectContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonObjectContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 58, RULE_jsonObject);
        let mut _localctx: Rc<JsonObjectContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(356);
			recog.base.match_token(Idl_LBrace,&mut recog.err_handler)?;

			recog.base.set_state(365);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Idl_StringLiteral {
				{
				/*InvokeRule jsonPair*/
				recog.base.set_state(357);
				let tmp = recog.jsonPair()?;
				 cast_mut::<_,JsonObjectContext >(&mut _localctx).jsonPair = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,JsonObjectContext >(&mut _localctx).jsonPair.clone().unwrap()
				 ;
				 cast_mut::<_,JsonObjectContext >(&mut _localctx).jsonPairs.push(temp);
				  
				recog.base.set_state(362);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==Idl_Comma {
					{
					{
					recog.base.set_state(358);
					recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

					/*InvokeRule jsonPair*/
					recog.base.set_state(359);
					let tmp = recog.jsonPair()?;
					 cast_mut::<_,JsonObjectContext >(&mut _localctx).jsonPair = Some(tmp.clone());
					  

					let temp =  cast_mut::<_,JsonObjectContext >(&mut _localctx).jsonPair.clone().unwrap()
					 ;
					 cast_mut::<_,JsonObjectContext >(&mut _localctx).jsonPairs.push(temp);
					  
					}
					}
					recog.base.set_state(364);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(367);
			recog.base.match_token(Idl_RBrace,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonPair ----------------
pub type JsonPairContextAll<'input> = JsonPairContext<'input>;


pub type JsonPairContext<'input> = BaseParserRuleContext<'input,JsonPairContextExt<'input>>;

#[derive(Clone)]
pub struct JsonPairContextExt<'input>{
	pub name: Option<TokenType<'input>>,
	pub value: Option<Rc<JsonValueContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for JsonPairContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for JsonPairContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonPair(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonPair(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for JsonPairContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonPair }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonPair }
}
antlr4rust::tid!{JsonPairContextExt<'a>}

impl<'input> JsonPairContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonPairContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonPairContextExt{
				name: None, 
				value: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonPairContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<JsonPairContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token Colon
/// Returns `None` if there is no child corresponding to token Colon
fn Colon(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Colon, 0)
}
/// Retrieves first TerminalNode corresponding to token StringLiteral
/// Returns `None` if there is no child corresponding to token StringLiteral
fn StringLiteral(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_StringLiteral, 0)
}
fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> JsonPairContextAttrs<'input> for JsonPairContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonPair(&mut self,)
	-> Result<Rc<JsonPairContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonPairContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 60, RULE_jsonPair);
        let mut _localctx: Rc<JsonPairContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(369);
			let tmp = recog.base.match_token(Idl_StringLiteral,&mut recog.err_handler)?;
			 cast_mut::<_,JsonPairContext >(&mut _localctx).name = Some(tmp.clone());
			  

			recog.base.set_state(370);
			recog.base.match_token(Idl_Colon,&mut recog.err_handler)?;

			/*InvokeRule jsonValue*/
			recog.base.set_state(371);
			let tmp = recog.jsonValue()?;
			 cast_mut::<_,JsonPairContext >(&mut _localctx).value = Some(tmp.clone());
			  

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonArray ----------------
pub type JsonArrayContextAll<'input> = JsonArrayContext<'input>;


pub type JsonArrayContext<'input> = BaseParserRuleContext<'input,JsonArrayContextExt<'input>>;

#[derive(Clone)]
pub struct JsonArrayContextExt<'input>{
	pub jsonValue: Option<Rc<JsonValueContextAll<'input>>>,
	pub jsonValues:Vec<Rc<JsonValueContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> IdlParserContext<'input> for JsonArrayContext<'input>{}

impl<'input,'a> Listenable<dyn IdlListener<'input> + 'a> for JsonArrayContext<'input>{
		fn enter(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonArray(self);
			Ok(())
		}fn exit(&self,listener: &mut (dyn IdlListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonArray(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input> CustomRuleContext<'input> for JsonArrayContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = IdlParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonArray }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonArray }
}
antlr4rust::tid!{JsonArrayContextExt<'a>}

impl<'input> JsonArrayContextExt<'input>{
	fn new(parent: Option<Rc<dyn IdlParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonArrayContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonArrayContextExt{
				jsonValue: None, 
				jsonValues: Vec::new(), 

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonArrayContextAttrs<'input>: IdlParserContext<'input> + BorrowMut<JsonArrayContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token LBracket
/// Returns `None` if there is no child corresponding to token LBracket
fn LBracket(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_LBracket, 0)
}
/// Retrieves first TerminalNode corresponding to token RBracket
/// Returns `None` if there is no child corresponding to token RBracket
fn RBracket(&self) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_RBracket, 0)
}
fn jsonValue_all(&self) ->  Vec<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn jsonValue(&self, i: usize) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token Comma in current rule
fn Comma_all(&self) -> Vec<Rc<TerminalNode<'input,IdlParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token Comma, starting from 0.
/// Returns `None` if number of children corresponding to token Comma is less or equal than `i`.
fn Comma(&self, i: usize) -> Option<Rc<TerminalNode<'input,IdlParserContextType>>> where Self:Sized{
	self.get_token(Idl_Comma, i)
}

}

impl<'input> JsonArrayContextAttrs<'input> for JsonArrayContext<'input>{}

impl<'input, I> IdlParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonArray(&mut self,)
	-> Result<Rc<JsonArrayContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonArrayContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 62, RULE_jsonArray);
        let mut _localctx: Rc<JsonArrayContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(373);
			recog.base.match_token(Idl_LBracket,&mut recog.err_handler)?;

			recog.base.set_state(382);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if ((((_la - 26)) & !0x3f) == 0 && ((1usize << (_la - 26)) & 1879130119) != 0) {
				{
				/*InvokeRule jsonValue*/
				recog.base.set_state(374);
				let tmp = recog.jsonValue()?;
				 cast_mut::<_,JsonArrayContext >(&mut _localctx).jsonValue = Some(tmp.clone());
				  

				let temp =  cast_mut::<_,JsonArrayContext >(&mut _localctx).jsonValue.clone().unwrap()
				 ;
				 cast_mut::<_,JsonArrayContext >(&mut _localctx).jsonValues.push(temp);
				  
				recog.base.set_state(379);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==Idl_Comma {
					{
					{
					recog.base.set_state(375);
					recog.base.match_token(Idl_Comma,&mut recog.err_handler)?;

					/*InvokeRule jsonValue*/
					recog.base.set_state(376);
					let tmp = recog.jsonValue()?;
					 cast_mut::<_,JsonArrayContext >(&mut _localctx).jsonValue = Some(tmp.clone());
					  

					let temp =  cast_mut::<_,JsonArrayContext >(&mut _localctx).jsonValue.clone().unwrap()
					 ;
					 cast_mut::<_,JsonArrayContext >(&mut _localctx).jsonValues.push(temp);
					  
					}
					}
					recog.base.set_state(381);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(384);
			recog.base.match_token(Idl_RBracket,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
	lazy_static!{
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(
                _ATN.clone(),
                _ATN.get_decision_state(i),
                i,
            ).into())
        }
        Arc::new(dfa)
    };
	static ref _serializedATN: Vec<i32> = vec![
		4, 1, 57, 387, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 
		4, 2, 5, 7, 5, 2, 6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 
		7, 10, 2, 11, 7, 11, 2, 12, 7, 12, 2, 13, 7, 13, 2, 14, 7, 14, 2, 15, 
		7, 15, 2, 16, 7, 16, 2, 17, 7, 17, 2, 18, 7, 18, 2, 19, 7, 19, 2, 20, 
		7, 20, 2, 21, 7, 21, 2, 22, 7, 22, 2, 23, 7, 23, 2, 24, 7, 24, 2, 25, 
		7, 25, 2, 26, 7, 26, 2, 27, 7, 27, 2, 28, 7, 28, 2, 29, 7, 29, 2, 30, 
		7, 30, 2, 31, 7, 31, 1, 0, 1, 0, 3, 0, 67, 8, 0, 1, 0, 3, 0, 70, 8, 0, 
		1, 0, 1, 0, 5, 0, 74, 8, 0, 10, 0, 12, 0, 77, 9, 0, 3, 0, 79, 8, 0, 1, 
		0, 1, 0, 5, 0, 83, 8, 0, 10, 0, 12, 0, 86, 9, 0, 3, 0, 88, 8, 0, 1, 0, 
		1, 0, 1, 1, 3, 1, 93, 8, 1, 1, 1, 5, 1, 96, 8, 1, 10, 1, 12, 1, 99, 9, 
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 1, 2, 1, 2, 5, 2, 109, 8, 2, 10, 
		2, 12, 2, 112, 9, 2, 1, 2, 1, 2, 1, 3, 1, 3, 1, 3, 1, 3, 1, 4, 1, 4, 1, 
		4, 1, 4, 1, 5, 1, 5, 1, 6, 1, 6, 1, 6, 1, 6, 1, 6, 1, 6, 1, 7, 1, 7, 1, 
		7, 1, 7, 1, 7, 1, 8, 1, 8, 1, 8, 3, 8, 140, 8, 8, 1, 9, 3, 9, 143, 8, 
		9, 1, 9, 5, 9, 146, 8, 9, 10, 9, 12, 9, 149, 9, 9, 1, 9, 1, 9, 1, 9, 1, 
		9, 1, 9, 1, 9, 1, 9, 1, 10, 3, 10, 159, 8, 10, 1, 10, 5, 10, 162, 8, 10, 
		10, 10, 12, 10, 165, 9, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 
		5, 10, 173, 8, 10, 10, 10, 12, 10, 176, 9, 10, 3, 10, 178, 8, 10, 1, 10, 
		1, 10, 3, 10, 182, 8, 10, 1, 11, 3, 11, 185, 8, 11, 1, 11, 5, 11, 188, 
		8, 11, 10, 11, 12, 11, 191, 9, 11, 1, 11, 1, 11, 1, 12, 1, 12, 1, 12, 
		1, 12, 1, 13, 3, 13, 200, 8, 13, 1, 13, 5, 13, 203, 8, 13, 10, 13, 12, 
		13, 206, 9, 13, 1, 13, 1, 13, 1, 13, 1, 13, 1, 14, 1, 14, 5, 14, 214, 
		8, 14, 10, 14, 12, 14, 217, 9, 14, 1, 14, 1, 14, 1, 15, 3, 15, 222, 8, 
		15, 1, 15, 1, 15, 1, 15, 1, 15, 5, 15, 228, 8, 15, 10, 15, 12, 15, 231, 
		9, 15, 1, 15, 1, 15, 1, 16, 3, 16, 236, 8, 16, 1, 16, 5, 16, 239, 8, 16, 
		10, 16, 12, 16, 242, 9, 16, 1, 16, 1, 16, 1, 16, 3, 16, 247, 8, 16, 1, 
		17, 3, 17, 250, 8, 17, 1, 17, 5, 17, 253, 8, 17, 10, 17, 12, 17, 256, 
		9, 17, 1, 17, 1, 17, 1, 17, 1, 17, 1, 17, 1, 17, 5, 17, 264, 8, 17, 10, 
		17, 12, 17, 267, 9, 17, 3, 17, 269, 8, 17, 1, 17, 1, 17, 1, 17, 1, 17, 
		1, 17, 1, 17, 5, 17, 277, 8, 17, 10, 17, 12, 17, 280, 9, 17, 3, 17, 282, 
		8, 17, 1, 17, 1, 17, 1, 18, 3, 18, 287, 8, 18, 1, 18, 1, 18, 1, 18, 1, 
		19, 1, 19, 3, 19, 294, 8, 19, 1, 20, 5, 20, 297, 8, 20, 10, 20, 12, 20, 
		300, 9, 20, 1, 20, 1, 20, 1, 21, 1, 21, 1, 21, 1, 21, 3, 21, 308, 8, 21, 
		1, 22, 1, 22, 3, 22, 312, 8, 22, 1, 22, 3, 22, 315, 8, 22, 1, 23, 1, 23, 
		1, 23, 1, 23, 1, 23, 1, 23, 3, 23, 323, 8, 23, 1, 23, 3, 23, 326, 8, 23, 
		1, 24, 1, 24, 1, 24, 1, 24, 1, 24, 1, 25, 1, 25, 1, 25, 1, 25, 1, 25, 
		1, 26, 1, 26, 1, 26, 1, 26, 1, 26, 5, 26, 343, 8, 26, 10, 26, 12, 26, 
		346, 9, 26, 1, 26, 1, 26, 1, 27, 1, 27, 1, 27, 3, 27, 353, 8, 27, 1, 28, 
		1, 28, 1, 29, 1, 29, 1, 29, 1, 29, 5, 29, 361, 8, 29, 10, 29, 12, 29, 
		364, 9, 29, 3, 29, 366, 8, 29, 1, 29, 1, 29, 1, 30, 1, 30, 1, 30, 1, 30, 
		1, 31, 1, 31, 1, 31, 1, 31, 5, 31, 378, 8, 31, 10, 31, 12, 31, 381, 9, 
		31, 3, 31, 383, 8, 31, 1, 31, 1, 31, 1, 31, 1, 84, 0, 32, 0, 2, 4, 6, 
		8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 
		44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 0, 5, 2, 0, 7, 37, 57, 57, 2, 
		0, 7, 7, 10, 11, 1, 0, 14, 15, 2, 0, 19, 26, 30, 34, 2, 0, 26, 28, 54, 
		56, 409, 0, 78, 1, 0, 0, 0, 2, 92, 1, 0, 0, 0, 4, 104, 1, 0, 0, 0, 6, 
		115, 1, 0, 0, 0, 8, 119, 1, 0, 0, 0, 10, 123, 1, 0, 0, 0, 12, 125, 1, 
		0, 0, 0, 14, 131, 1, 0, 0, 0, 16, 139, 1, 0, 0, 0, 18, 142, 1, 0, 0, 0, 
		20, 158, 1, 0, 0, 0, 22, 184, 1, 0, 0, 0, 24, 194, 1, 0, 0, 0, 26, 199, 
		1, 0, 0, 0, 28, 211, 1, 0, 0, 0, 30, 221, 1, 0, 0, 0, 32, 235, 1, 0, 0, 
		0, 34, 249, 1, 0, 0, 0, 36, 286, 1, 0, 0, 0, 38, 293, 1, 0, 0, 0, 40, 
		298, 1, 0, 0, 0, 42, 307, 1, 0, 0, 0, 44, 311, 1, 0, 0, 0, 46, 325, 1, 
		0, 0, 0, 48, 327, 1, 0, 0, 0, 50, 332, 1, 0, 0, 0, 52, 337, 1, 0, 0, 0, 
		54, 352, 1, 0, 0, 0, 56, 354, 1, 0, 0, 0, 58, 356, 1, 0, 0, 0, 60, 369, 
		1, 0, 0, 0, 62, 373, 1, 0, 0, 0, 64, 79, 3, 2, 1, 0, 65, 67, 3, 6, 3, 
		0, 66, 65, 1, 0, 0, 0, 66, 67, 1, 0, 0, 0, 67, 69, 1, 0, 0, 0, 68, 70, 
		3, 8, 4, 0, 69, 68, 1, 0, 0, 0, 69, 70, 1, 0, 0, 0, 70, 75, 1, 0, 0, 0, 
		71, 74, 3, 14, 7, 0, 72, 74, 3, 16, 8, 0, 73, 71, 1, 0, 0, 0, 73, 72, 
		1, 0, 0, 0, 74, 77, 1, 0, 0, 0, 75, 73, 1, 0, 0, 0, 75, 76, 1, 0, 0, 0, 
		76, 79, 1, 0, 0, 0, 77, 75, 1, 0, 0, 0, 78, 64, 1, 0, 0, 0, 78, 66, 1, 
		0, 0, 0, 79, 87, 1, 0, 0, 0, 80, 84, 5, 1, 0, 0, 81, 83, 9, 0, 0, 0, 82, 
		81, 1, 0, 0, 0, 83, 86, 1, 0, 0, 0, 84, 85, 1, 0, 0, 0, 84, 82, 1, 0, 
		0, 0, 85, 88, 1, 0, 0, 0, 86, 84, 1, 0, 0, 0, 87, 80, 1, 0, 0, 0, 87, 
		88, 1, 0, 0, 0, 88, 89, 1, 0, 0, 0, 89, 90, 5, 0, 0, 1, 90, 1, 1, 0, 0, 
		0, 91, 93, 5, 2, 0, 0, 92, 91, 1, 0, 0, 0, 92, 93, 1, 0, 0, 0, 93, 97, 
		1, 0, 0, 0, 94, 96, 3, 12, 6, 0, 95, 94, 1, 0, 0, 0, 96, 99, 1, 0, 0, 
		0, 97, 95, 1, 0, 0, 0, 97, 98, 1, 0, 0, 0, 98, 100, 1, 0, 0, 0, 99, 97, 
		1, 0, 0, 0, 100, 101, 5, 7, 0, 0, 101, 102, 3, 10, 5, 0, 102, 103, 3, 
		4, 2, 0, 103, 3, 1, 0, 0, 0, 104, 110, 5, 40, 0, 0, 105, 109, 3, 14, 7, 
		0, 106, 109, 3, 16, 8, 0, 107, 109, 3, 34, 17, 0, 108, 105, 1, 0, 0, 0, 
		108, 106, 1, 0, 0, 0, 108, 107, 1, 0, 0, 0, 109, 112, 1, 0, 0, 0, 110, 
		108, 1, 0, 0, 0, 110, 111, 1, 0, 0, 0, 111, 113, 1, 0, 0, 0, 112, 110, 
		1, 0, 0, 0, 113, 114, 5, 41, 0, 0, 114, 5, 1, 0, 0, 0, 115, 116, 5, 8, 
		0, 0, 116, 117, 3, 10, 5, 0, 117, 118, 5, 45, 0, 0, 118, 7, 1, 0, 0, 0, 
		119, 120, 5, 11, 0, 0, 120, 121, 3, 40, 20, 0, 121, 122, 5, 45, 0, 0, 
		122, 9, 1, 0, 0, 0, 123, 124, 7, 0, 0, 0, 124, 11, 1, 0, 0, 0, 125, 126, 
		5, 47, 0, 0, 126, 127, 3, 10, 5, 0, 127, 128, 5, 38, 0, 0, 128, 129, 3, 
		54, 27, 0, 129, 130, 5, 39, 0, 0, 130, 13, 1, 0, 0, 0, 131, 132, 5, 9, 
		0, 0, 132, 133, 7, 1, 0, 0, 133, 134, 5, 54, 0, 0, 134, 135, 5, 45, 0, 
		0, 135, 15, 1, 0, 0, 0, 136, 140, 3, 18, 9, 0, 137, 140, 3, 20, 10, 0, 
		138, 140, 3, 26, 13, 0, 139, 136, 1, 0, 0, 0, 139, 137, 1, 0, 0, 0, 139, 
		138, 1, 0, 0, 0, 140, 17, 1, 0, 0, 0, 141, 143, 5, 2, 0, 0, 142, 141, 
		1, 0, 0, 0, 142, 143, 1, 0, 0, 0, 143, 147, 1, 0, 0, 0, 144, 146, 3, 12, 
		6, 0, 145, 144, 1, 0, 0, 0, 146, 149, 1, 0, 0, 0, 147, 145, 1, 0, 0, 0, 
		147, 148, 1, 0, 0, 0, 148, 150, 1, 0, 0, 0, 149, 147, 1, 0, 0, 0, 150, 
		151, 5, 13, 0, 0, 151, 152, 3, 10, 5, 0, 152, 153, 5, 38, 0, 0, 153, 154, 
		5, 55, 0, 0, 154, 155, 5, 39, 0, 0, 155, 156, 5, 45, 0, 0, 156, 19, 1, 
		0, 0, 0, 157, 159, 5, 2, 0, 0, 158, 157, 1, 0, 0, 0, 158, 159, 1, 0, 0, 
		0, 159, 163, 1, 0, 0, 0, 160, 162, 3, 12, 6, 0, 161, 160, 1, 0, 0, 0, 
		162, 165, 1, 0, 0, 0, 163, 161, 1, 0, 0, 0, 163, 164, 1, 0, 0, 0, 164, 
		166, 1, 0, 0, 0, 165, 163, 1, 0, 0, 0, 166, 167, 5, 12, 0, 0, 167, 168, 
		3, 10, 5, 0, 168, 177, 5, 40, 0, 0, 169, 174, 3, 22, 11, 0, 170, 171, 
		5, 46, 0, 0, 171, 173, 3, 22, 11, 0, 172, 170, 1, 0, 0, 0, 173, 176, 1, 
		0, 0, 0, 174, 172, 1, 0, 0, 0, 174, 175, 1, 0, 0, 0, 175, 178, 1, 0, 0, 
		0, 176, 174, 1, 0, 0, 0, 177, 169, 1, 0, 0, 0, 177, 178, 1, 0, 0, 0, 178, 
		179, 1, 0, 0, 0, 179, 181, 5, 41, 0, 0, 180, 182, 3, 24, 12, 0, 181, 180, 
		1, 0, 0, 0, 181, 182, 1, 0, 0, 0, 182, 21, 1, 0, 0, 0, 183, 185, 5, 2, 
		0, 0, 184, 183, 1, 0, 0, 0, 184, 185, 1, 0, 0, 0, 185, 189, 1, 0, 0, 0, 
		186, 188, 3, 12, 6, 0, 187, 186, 1, 0, 0, 0, 188, 191, 1, 0, 0, 0, 189, 
		187, 1, 0, 0, 0, 189, 190, 1, 0, 0, 0, 190, 192, 1, 0, 0, 0, 191, 189, 
		1, 0, 0, 0, 192, 193, 3, 10, 5, 0, 193, 23, 1, 0, 0, 0, 194, 195, 5, 48, 
		0, 0, 195, 196, 3, 10, 5, 0, 196, 197, 5, 45, 0, 0, 197, 25, 1, 0, 0, 
		0, 198, 200, 5, 2, 0, 0, 199, 198, 1, 0, 0, 0, 199, 200, 1, 0, 0, 0, 200, 
		204, 1, 0, 0, 0, 201, 203, 3, 12, 6, 0, 202, 201, 1, 0, 0, 0, 203, 206, 
		1, 0, 0, 0, 204, 202, 1, 0, 0, 0, 204, 205, 1, 0, 0, 0, 205, 207, 1, 0, 
		0, 0, 206, 204, 1, 0, 0, 0, 207, 208, 7, 2, 0, 0, 208, 209, 3, 10, 5, 
		0, 209, 210, 3, 28, 14, 0, 210, 27, 1, 0, 0, 0, 211, 215, 5, 40, 0, 0, 
		212, 214, 3, 30, 15, 0, 213, 212, 1, 0, 0, 0, 214, 217, 1, 0, 0, 0, 215, 
		213, 1, 0, 0, 0, 215, 216, 1, 0, 0, 0, 216, 218, 1, 0, 0, 0, 217, 215, 
		1, 0, 0, 0, 218, 219, 5, 41, 0, 0, 219, 29, 1, 0, 0, 0, 220, 222, 5, 2, 
		0, 0, 221, 220, 1, 0, 0, 0, 221, 222, 1, 0, 0, 0, 222, 223, 1, 0, 0, 0, 
		223, 224, 3, 40, 20, 0, 224, 229, 3, 32, 16, 0, 225, 226, 5, 46, 0, 0, 
		226, 228, 3, 32, 16, 0, 227, 225, 1, 0, 0, 0, 228, 231, 1, 0, 0, 0, 229, 
		227, 1, 0, 0, 0, 229, 230, 1, 0, 0, 0, 230, 232, 1, 0, 0, 0, 231, 229, 
		1, 0, 0, 0, 232, 233, 5, 45, 0, 0, 233, 31, 1, 0, 0, 0, 234, 236, 5, 2, 
		0, 0, 235, 234, 1, 0, 0, 0, 235, 236, 1, 0, 0, 0, 236, 240, 1, 0, 0, 0, 
		237, 239, 3, 12, 6, 0, 238, 237, 1, 0, 0, 0, 239, 242, 1, 0, 0, 0, 240, 
		238, 1, 0, 0, 0, 240, 241, 1, 0, 0, 0, 241, 243, 1, 0, 0, 0, 242, 240, 
		1, 0, 0, 0, 243, 246, 3, 10, 5, 0, 244, 245, 5, 48, 0, 0, 245, 247, 3, 
		54, 27, 0, 246, 244, 1, 0, 0, 0, 246, 247, 1, 0, 0, 0, 247, 33, 1, 0, 
		0, 0, 248, 250, 5, 2, 0, 0, 249, 248, 1, 0, 0, 0, 249, 250, 1, 0, 0, 0, 
		250, 254, 1, 0, 0, 0, 251, 253, 3, 12, 6, 0, 252, 251, 1, 0, 0, 0, 253, 
		256, 1, 0, 0, 0, 254, 252, 1, 0, 0, 0, 254, 255, 1, 0, 0, 0, 255, 257, 
		1, 0, 0, 0, 256, 254, 1, 0, 0, 0, 257, 258, 3, 38, 19, 0, 258, 259, 3, 
		10, 5, 0, 259, 268, 5, 38, 0, 0, 260, 265, 3, 36, 18, 0, 261, 262, 5, 
		46, 0, 0, 262, 264, 3, 36, 18, 0, 263, 261, 1, 0, 0, 0, 264, 267, 1, 0, 
		0, 0, 265, 263, 1, 0, 0, 0, 265, 266, 1, 0, 0, 0, 266, 269, 1, 0, 0, 0, 
		267, 265, 1, 0, 0, 0, 268, 260, 1, 0, 0, 0, 268, 269, 1, 0, 0, 0, 269, 
		270, 1, 0, 0, 0, 270, 281, 5, 39, 0, 0, 271, 282, 5, 36, 0, 0, 272, 273, 
		5, 37, 0, 0, 273, 278, 3, 10, 5, 0, 274, 275, 5, 46, 0, 0, 275, 277, 3, 
		10, 5, 0, 276, 274, 1, 0, 0, 0, 277, 280, 1, 0, 0, 0, 278, 276, 1, 0, 
		0, 0, 278, 279, 1, 0, 0, 0, 279, 282, 1, 0, 0, 0, 280, 278, 1, 0, 0, 0, 
		281, 271, 1, 0, 0, 0, 281, 272, 1, 0, 0, 0, 281, 282, 1, 0, 0, 0, 282, 
		283, 1, 0, 0, 0, 283, 284, 5, 45, 0, 0, 284, 35, 1, 0, 0, 0, 285, 287, 
		5, 2, 0, 0, 286, 285, 1, 0, 0, 0, 286, 287, 1, 0, 0, 0, 287, 288, 1, 0, 
		0, 0, 288, 289, 3, 40, 20, 0, 289, 290, 3, 32, 16, 0, 290, 37, 1, 0, 0, 
		0, 291, 294, 5, 35, 0, 0, 292, 294, 3, 42, 21, 0, 293, 291, 1, 0, 0, 0, 
		293, 292, 1, 0, 0, 0, 294, 39, 1, 0, 0, 0, 295, 297, 3, 12, 6, 0, 296, 
		295, 1, 0, 0, 0, 297, 300, 1, 0, 0, 0, 298, 296, 1, 0, 0, 0, 298, 299, 
		1, 0, 0, 0, 299, 301, 1, 0, 0, 0, 300, 298, 1, 0, 0, 0, 301, 302, 3, 42, 
		21, 0, 302, 41, 1, 0, 0, 0, 303, 308, 3, 48, 24, 0, 304, 308, 3, 50, 25, 
		0, 305, 308, 3, 52, 26, 0, 306, 308, 3, 44, 22, 0, 307, 303, 1, 0, 0, 
		0, 307, 304, 1, 0, 0, 0, 307, 305, 1, 0, 0, 0, 307, 306, 1, 0, 0, 0, 308, 
		43, 1, 0, 0, 0, 309, 312, 3, 46, 23, 0, 310, 312, 3, 10, 5, 0, 311, 309, 
		1, 0, 0, 0, 311, 310, 1, 0, 0, 0, 312, 314, 1, 0, 0, 0, 313, 315, 5, 51, 
		0, 0, 314, 313, 1, 0, 0, 0, 314, 315, 1, 0, 0, 0, 315, 45, 1, 0, 0, 0, 
		316, 326, 7, 3, 0, 0, 317, 318, 5, 29, 0, 0, 318, 319, 5, 38, 0, 0, 319, 
		322, 5, 55, 0, 0, 320, 321, 5, 46, 0, 0, 321, 323, 5, 55, 0, 0, 322, 320, 
		1, 0, 0, 0, 322, 323, 1, 0, 0, 0, 323, 324, 1, 0, 0, 0, 324, 326, 5, 39, 
		0, 0, 325, 316, 1, 0, 0, 0, 325, 317, 1, 0, 0, 0, 326, 47, 1, 0, 0, 0, 
		327, 328, 5, 16, 0, 0, 328, 329, 5, 52, 0, 0, 329, 330, 3, 40, 20, 0, 
		330, 331, 5, 53, 0, 0, 331, 49, 1, 0, 0, 0, 332, 333, 5, 17, 0, 0, 333, 
		334, 5, 52, 0, 0, 334, 335, 3, 40, 20, 0, 335, 336, 5, 53, 0, 0, 336, 
		51, 1, 0, 0, 0, 337, 338, 5, 18, 0, 0, 338, 339, 5, 40, 0, 0, 339, 344, 
		3, 40, 20, 0, 340, 341, 5, 46, 0, 0, 341, 343, 3, 40, 20, 0, 342, 340, 
		1, 0, 0, 0, 343, 346, 1, 0, 0, 0, 344, 342, 1, 0, 0, 0, 344, 345, 1, 0, 
		0, 0, 345, 347, 1, 0, 0, 0, 346, 344, 1, 0, 0, 0, 347, 348, 5, 41, 0, 
		0, 348, 53, 1, 0, 0, 0, 349, 353, 3, 58, 29, 0, 350, 353, 3, 62, 31, 0, 
		351, 353, 3, 56, 28, 0, 352, 349, 1, 0, 0, 0, 352, 350, 1, 0, 0, 0, 352, 
		351, 1, 0, 0, 0, 353, 55, 1, 0, 0, 0, 354, 355, 7, 4, 0, 0, 355, 57, 1, 
		0, 0, 0, 356, 365, 5, 40, 0, 0, 357, 362, 3, 60, 30, 0, 358, 359, 5, 46, 
		0, 0, 359, 361, 3, 60, 30, 0, 360, 358, 1, 0, 0, 0, 361, 364, 1, 0, 0, 
		0, 362, 360, 1, 0, 0, 0, 362, 363, 1, 0, 0, 0, 363, 366, 1, 0, 0, 0, 364, 
		362, 1, 0, 0, 0, 365, 357, 1, 0, 0, 0, 365, 366, 1, 0, 0, 0, 366, 367, 
		1, 0, 0, 0, 367, 368, 5, 41, 0, 0, 368, 59, 1, 0, 0, 0, 369, 370, 5, 54, 
		0, 0, 370, 371, 5, 44, 0, 0, 371, 372, 3, 54, 27, 0, 372, 61, 1, 0, 0, 
		0, 373, 382, 5, 42, 0, 0, 374, 379, 3, 54, 27, 0, 375, 376, 5, 46, 0, 
		0, 376, 378, 3, 54, 27, 0, 377, 375, 1, 0, 0, 0, 378, 381, 1, 0, 0, 0, 
		379, 377, 1, 0, 0, 0, 379, 380, 1, 0, 0, 0, 380, 383, 1, 0, 0, 0, 381, 
		379, 1, 0, 0, 0, 382, 374, 1, 0, 0, 0, 382, 383, 1, 0, 0, 0, 383, 384, 
		1, 0, 0, 0, 384, 385, 5, 43, 0, 0, 385, 63, 1, 0, 0, 0, 49, 66, 69, 73, 
		75, 78, 84, 87, 92, 97, 108, 110, 139, 142, 147, 158, 163, 174, 177, 181, 
		184, 189, 199, 204, 215, 221, 229, 235, 240, 246, 249, 254, 265, 268, 
		278, 281, 286, 293, 298, 307, 311, 314, 322, 325, 344, 352, 362, 365, 
		379, 382
	];
}
