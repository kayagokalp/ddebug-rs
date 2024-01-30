//! Code generation from given `AbstractSyntaxTree`.

use thiserror::Error;

use crate::{graph::SyntaxTree, parser::AstNode};

/// Code generation from the `SyntaxTree`.
pub struct CodeGenerator<'a> {
    syntax_tree: &'a SyntaxTree<'a>,
}

#[derive(Debug, Error)]
pub enum CodeGeneratorError {
    #[error("Missing root node in the syntax tree")]
    RootNodeMissingInSyntaxTree,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(syntax_tree: &'a SyntaxTree<'a>) -> Self {
        Self { syntax_tree }
    }

    pub fn generate(self) -> Result<String, CodeGeneratorError> {
        // Get the source root.
        let root_node_ix = self
            .syntax_tree
            .root_node()
            .ok_or(CodeGeneratorError::RootNodeMissingInSyntaxTree)?;

        let graph = self.syntax_tree.as_ref();
        if let AstNode::SourceRoot(file) = graph[root_node_ix] {
            Ok(prettyplease::unparse(file))
        } else {
            Err(CodeGeneratorError::RootNodeMissingInSyntaxTree)
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::visit::Visit;

    use crate::{
        graph::{GraphBuilder, SyntaxTree},
        parser::AbstractSyntaxTree,
    };

    use super::CodeGenerator;

    #[test]
    fn parse_unparse_parse() {
        let test_code = r#"
fn test_fn() {}
fn main() {}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        let file = parsed_ast.clone().syn_file();

        let mut syntax_tree = SyntaxTree::new();
        let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None);
        graph_builder.visit_file(&file);

        let code_generator = CodeGenerator::new(graph_builder.syntax_tree());
        let generated_code = code_generator.generate().unwrap();

        let reparsed_ast = AbstractSyntaxTree::parse(generated_code);

        assert_eq!(parsed_ast, reparsed_ast)
    }
}
