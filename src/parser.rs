use syn::{Block, ExprArray, ExprAssign, ExprLet, File, Item, ItemFn, Local};

#[derive(Debug, Eq, PartialEq, Clone)]
/// Representation of the syntax generated from parsing a rust code piece.
pub struct AbstractSyntaxTree {
    pub attributes: Vec<syn::Attribute>,
    pub items: Vec<syn::Item>,
}

#[derive(Eq, PartialEq, Clone)]
// Custom enum to represent possible AST nodes
pub enum AstNode<'a> {
    SourceRoot(&'a File),
    Item(&'a Item),
    ItemFn(&'a ItemFn),
    Block(&'a Block),
    LocalStmt(&'a Local),
    ExprArray(&'a ExprArray),
    ExprAssign(&'a ExprAssign),
    ExprLet(&'a ExprLet),
}

impl AbstractSyntaxTree {
    /// Parse a given str into an AST representation.
    pub fn parse<T: AsRef<str>>(input: T) -> Self {
        let syntax = syn::parse_str::<syn::File>(input.as_ref()).unwrap();

        Self {
            attributes: syntax.attrs,
            items: syntax.items,
        }
    }

    /// Returns the abstract syntax tree as a syn `File`.
    ///
    /// Note: syn file representation contains shebang field. For file's genereted from AST,
    /// shebang is returned as `None`.
    pub fn syn_file(self) -> syn::File {
        syn::File {
            shebang: None,
            attrs: self.attributes,
            items: self.items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AbstractSyntaxTree;

    #[test]
    fn parser_empty_string() {
        let input_str = "";

        let parsed_ast = AbstractSyntaxTree::parse(input_str);
        let expected_ast = AbstractSyntaxTree {
            attributes: vec![],
            items: vec![],
        };

        assert_eq!(parsed_ast, expected_ast)
    }

    #[test]
    fn parser_single_item_without_attribute() {
        let test_code = r#"
fn main() {}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        assert_eq!(parsed_ast.items.len(), 1);
    }

    #[test]
    fn parser_multiple_item_without_attribute() {
        let test_code = r#"
fn test_fn() {}
fn main() {}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        assert_eq!(parsed_ast.items.len(), 2);
    }
}
