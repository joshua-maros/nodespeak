use super::{Content, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::structure::{BaseType, DataType, Expression, KnownData};

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn simplify_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let simplified_target = self.simplify_assignment_access_expression(target)?;
        let simplified_value = self.simplify_expression(value)?;
        if simplified_target.1.is_automatic() {
            self.simplify_automatic_type(&simplified_target.0, &simplified_value.data_type)?;
        } else if !simplified_value
            .data_type
            .equivalent(&simplified_target.1, self.program)
        {
            panic!("TODO: nice error, mismatched data types in assignment.");
        }
        Result::Ok(match simplified_value.content {
            Content::Interpreted(known_value) => {
                let result = self.assign_value_to_expression(
                    &simplified_target.0,
                    known_value,
                    position.clone(),
                )?;
                if let Result::Err(simplified_expresion) = result {
                    SimplifiedExpression {
                        content: Content::Modified(simplified_expresion),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                } else {
                    SimplifiedExpression {
                        content: Content::Interpreted(KnownData::Void),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                }
            }
            Content::Modified(simplified_value) => {
                self.assign_unknown_to_expression(&simplified_target.0);
                let content = Content::Modified(Expression::Assign {
                    target: Box::new(simplified_target.0),
                    value: Box::new(simplified_value),
                    position: position.clone(),
                });
                SimplifiedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }
        })
    }
}
