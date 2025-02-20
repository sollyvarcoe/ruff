use crate::comments::{dangling_comments, CommentTextPosition, Comments};
use crate::expression::parentheses::{
    default_expression_needs_parentheses, NeedsParentheses, Parentheses, Parenthesize,
};
use crate::prelude::*;
use crate::FormatNodeRule;
use ruff_formatter::{format_args, write};
use rustpython_parser::ast::ExprList;

#[derive(Default)]
pub struct FormatExprList;

impl FormatNodeRule<ExprList> for FormatExprList {
    fn fmt_fields(&self, item: &ExprList, f: &mut PyFormatter) -> FormatResult<()> {
        let ExprList {
            range: _,
            elts,
            ctx: _,
        } = item;

        let comments = f.context().comments().clone();
        let dangling = comments.dangling_comments(item.into());

        // The empty list is special because there can be dangling comments, and they can be in two
        // positions:
        // ```python
        // a3 = [  # end-of-line
        //     # own line
        // ]
        // ```
        // In all other cases comments get assigned to a list element
        if elts.is_empty() {
            let end_of_line_split = dangling
                .partition_point(|comment| comment.position() == CommentTextPosition::EndOfLine);
            debug_assert!(dangling[end_of_line_split..]
                .iter()
                .all(|comment| comment.position() == CommentTextPosition::OwnLine));
            return write!(
                f,
                [group(&format_args![
                    text("["),
                    dangling_comments(&dangling[..end_of_line_split]),
                    soft_block_indent(&dangling_comments(&dangling[end_of_line_split..])),
                    text("]")
                ])]
            );
        }

        debug_assert!(
            dangling.is_empty(),
            "A non-empty expression list has dangling comments"
        );

        let items = format_with(|f| {
            let mut iter = elts.iter();

            if let Some(first) = iter.next() {
                write!(f, [first.format()])?;
            }

            for item in iter {
                write!(f, [text(","), soft_line_break_or_space(), item.format()])?;
            }

            if !elts.is_empty() {
                write!(f, [if_group_breaks(&text(","))])?;
            }

            Ok(())
        });

        write!(
            f,
            [group(&format_args![
                text("["),
                soft_block_indent(&items),
                text("]")
            ])]
        )
    }

    fn fmt_dangling_comments(&self, _node: &ExprList, _f: &mut PyFormatter) -> FormatResult<()> {
        // Handled as part of `fmt_fields`
        Ok(())
    }
}

impl NeedsParentheses for ExprList {
    fn needs_parentheses(
        &self,
        parenthesize: Parenthesize,
        source: &str,
        comments: &Comments,
    ) -> Parentheses {
        match default_expression_needs_parentheses(self.into(), parenthesize, source, comments) {
            Parentheses::Optional => Parentheses::Never,
            parentheses => parentheses,
        }
    }
}
