use std::ops::Range;
use tree_sitter::{Parser, Point, Node, Tree};

pub mod code_repository;

fn has_intersection(first: Range<usize>, second: Range<usize>) -> bool {
    second.contains(&first.start) || first.contains(&second.start)
}

pub struct CodeRegion {
    code: String,
    tree: Tree,

}

impl CodeRegion{
    pub fn new(code: &str) -> CodeRegion {

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c::language()).expect("Error loading C grammar");
        let tree = parser.parse(code, None).unwrap();

        CodeRegion{
            code: code.into(),
            tree: tree,
        }
    }

    fn extract_function_next_from_range(&self, range: Range<usize>) -> Option<Node>{
        let mut cursor = self.tree.walk();
        cursor.goto_first_child_for_point(Point::new(range.start, 0));

        let current_node = cursor.node();
        dbg!(&range);
        dbg!(current_node);
        let start_line = current_node.start_position().row;
        let end_line = current_node.end_position().row;

        if has_intersection(range.clone(), start_line..end_line) {
            Some(current_node.clone())
        } else {
            None
        }
    }

    fn extract_function_from_node(&self, function_node: Node) -> String {
        let start = function_node.start_byte();
        let end = function_node.end_byte();
        String::from_utf8_lossy(&self.code.as_bytes()[start..end]).to_string()
    }

    pub fn extract_functions(&self, range: Range<usize>) -> Vec<String> {

        let mut functions = vec![];

        let mut next_range = range.clone();
        while !self.code.is_empty() && !next_range.is_empty() {
            match self.extract_function_next_from_range(next_range.clone()) {
                Some(function) => {
                    functions.push(self.extract_function_from_node(function));
                    next_range = (function.range().end_point.row+1)..next_range.end;
                },
                None => break
            }
        }
        functions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn get_empty_vec_from_empty_content_test() {
        let code = CodeRegion::new("");
        assert!(code.extract_functions(0..1).is_empty());
    }

    #[test]
    fn get_function_if_content_contains_single_function_test() {
        let content = "int main(int argc, char** argv) {return 0;}";
        let code = CodeRegion::new(&content);
        assert!(!code.extract_functions(0..1).is_empty())
    }

    #[test]
    fn get_function_if_content_contains_single_function_and_region_is_empty_test() {
        let content = "int main(int argc, char** argv) {return 0;}";
        let code = CodeRegion::new(&content);
        assert!(code.extract_functions(1..1).is_empty())
    }

    #[test]
    fn get_single_line_function_from_multi_function_content() {
        let content = indoc!{"
        void foo() {}
        int main() {foo()};
        "};
        let all_functions = CodeRegion::new(&content).extract_functions(0..1);
        let functions_containing_main = all_functions.iter().find(|c| c.contains("main"));
        assert!(functions_containing_main.is_none());
    }

    #[test]
    fn get_function_from_multi_function_content() {
        let content = indoc!{r#"
        #include <stdio.h>
        void foo() {
            println("foo")
        }
        int main() {foo()};
        "#};
        let all_functions = CodeRegion::new(&content).extract_functions(2..4);
        let functions_containing_main = all_functions.iter().find(|c| c.contains("main"));
        assert!(functions_containing_main.is_none());
    }

    #[test]
    fn get_multi_line_function_with_narrow_range() {
        let content = indoc!{r#"
        #include <stdio.h>
        void foo() {
            println("foo")
        }
        int main() {foo()};
        "#};
        let all_functions = CodeRegion::new(&content).extract_functions(2..3);
        let functions_containing_main = all_functions.iter().find(|c| c.contains("void foo()"));
        assert!(functions_containing_main.is_some());
    }

    #[test]
    fn get_multiple_functions_with_wider_range() {
        let content = indoc!{r#"
        #include <stdio.h>
        void foo() {
            println("foo")
        }
        int main() {foo()};
        "#};
        let all_functions = CodeRegion::new(&content).extract_functions(2..5);
        assert!(all_functions.len() == 2);
    }

    #[test]
    fn empty_empty_has_no_intersection_test() {
        assert!(!has_intersection(0..0, 0..0))
    }

    #[test]
    fn first_is_contained_in_second() {
        assert!(has_intersection(1..2, 0..3))
    }


    #[test]
    fn second_is_contained_in_first() {
        assert!(has_intersection(0..3, 1..2))
    }

    #[test]
    fn left_overlap() {
        assert!(has_intersection(0..2, 1..3))
    }

    #[test]
    fn right_overlap() {
        assert!(has_intersection(1..3, 0..2))
    }

    #[test]
    fn left_non_overlap_test() {
        assert!(!has_intersection(0..3, 3..8))
    }

    #[test]
    fn right_non_overlap_test() {
        assert!(!has_intersection(3..8, 0..3))
    }

}
