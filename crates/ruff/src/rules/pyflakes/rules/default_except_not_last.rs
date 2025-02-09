use rustpython_parser::ast::{self, Excepthandler};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::helpers::except_range;
use ruff_python_ast::source_code::Locator;

/// ## What it does
/// Checks for `except` blocks that handle all exceptions, but are not the last
/// `except` block in a `try` statement.
///
/// ## Why is this bad?
/// When an exception is raised within a `try` block, the `except` blocks are
/// evaluated in order, and the first matching block is executed. If an `except`
/// block handles all exceptions, but isn't the last block, Python will raise a
/// `SyntaxError`, as the following blocks would never be executed.
///
/// ## Example
/// ```python
/// def reciprocal(n):
///     try:
///         reciprocal = 1 / n
///     except:
///         print("An exception occurred.")
///     except ZeroDivisionError:
///         print("Cannot divide by zero.")
///     else:
///         return reciprocal
/// ```
///
/// Use instead:
/// ```python
/// def reciprocal(n):
///     try:
///         reciprocal = 1 / n
///     except ZeroDivisionError:
///         print("Cannot divide by zero.")
///     except:
///         print("An exception occurred.")
///     else:
///         return reciprocal
/// ```
///
/// ## References
/// - [Python documentation: `except` clause](https://docs.python.org/3/reference/compound_stmts.html#except-clause)
#[violation]
pub struct DefaultExceptNotLast;

impl Violation for DefaultExceptNotLast {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("An `except` block as not the last exception handler")
    }
}

/// F707
pub(crate) fn default_except_not_last(
    handlers: &[Excepthandler],
    locator: &Locator,
) -> Option<Diagnostic> {
    for (idx, handler) in handlers.iter().enumerate() {
        let Excepthandler::ExceptHandler(ast::ExcepthandlerExceptHandler { type_, .. }) = handler;
        if type_.is_none() && idx < handlers.len() - 1 {
            return Some(Diagnostic::new(
                DefaultExceptNotLast,
                except_range(handler, locator),
            ));
        }
    }

    None
}
