#pragma once

#include <boost/fusion/include/adapt_struct.hpp>

#include "ast.hpp"

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::FunctionExpression,
    functionName, inputs, outputs
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::OperatorExpression,
    op_char, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::SignedExpression,
    sign, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::VariableExpression,
    name, array_accesses
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::OperatorListExpression,
    start_value, operations
)



BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::DataType,
    name, array_sizes
)



BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::FunctionStatement,
    func_call
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::AssignStatement,
    assign_to, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::PlainVarDec,
    name
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::InitVarDec,
    name, value
)

BOOST_FUSION_ADAPT_STRUCT(
    waveguide::ast::VarDecStatement,
    type, var_decs
)
