#pragma once

#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>
#include <string>
#include <vector>

namespace waveguide {
namespace ast {

namespace x3 = boost::spirit::x3;

struct FunctionStatement;
struct AssignStatement;
struct VarDecStatement;

using StatementVariant = x3::variant<
    x3::forward_ast<FunctionStatement>,
    x3::forward_ast<AssignStatement>,
    x3::forward_ast<VarDecStatement>>;
struct Statement: StatementVariant, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(Statement const&stat) { base_type::operator=(stat); }
    Statement(Statement &stat) : StatementVariant(stat) { }
    Statement(Statement const&stat) : StatementVariant(stat) { }
};

struct FunctionExpression;
struct OperatorListExpression;
struct SignedExpression;
struct VariableExpression;

struct Expression: x3::variant<
    int, double, bool, 
    x3::forward_ast<FunctionExpression>, 
    x3::forward_ast<VariableExpression>,
    x3::forward_ast<OperatorListExpression>, 
    x3::forward_ast<SignedExpression>>, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
};



struct DataType: x3::position_tagged {
    std::string name;
    std::vector<Expression> array_sizes;
};



struct FunctionInputDec: x3::position_tagged {
    DataType type;
    std::string name;
};

struct FunctionDec: x3::position_tagged {
    std::string name;
    std::vector<FunctionInputDec> inputs, outputs;
    std::vector<x3::forward_ast<FunctionDec>> lambdas;
    std::vector<Statement> body;
};



struct OperatorExpression: x3::position_tagged {
    std::string op_char;
    Expression value;
};

struct OperatorListExpression: x3::position_tagged {
    Expression start_value;
    std::vector<OperatorExpression> operations;
};

struct SignedExpression: x3::position_tagged {
    char sign;
    Expression value;
};

struct VariableExpression: x3::position_tagged {
    std::string name;
    std::vector<Expression> array_accesses;
};

struct FunctionExpression: x3::position_tagged {
    std::string functionName;
    std::vector<Expression> inputs;
    std::vector<VariableExpression> outputs;
    std::vector<FunctionDec> lambdas;
};



struct FunctionStatement: x3::position_tagged {
    FunctionExpression func_call;
};

struct AssignStatement: x3::position_tagged {
    VariableExpression assign_to;
    Expression value;
};

struct PlainVarDec: x3::position_tagged {
    std::string name;
};

struct InitVarDec: x3::position_tagged {
    std::string name;
    Expression value;
};

struct VarDec: x3::variant<PlainVarDec, InitVarDec>, x3::position_tagged {
    using base_type::base_type;
    using base_type::operator=;
    void operator=(VarDec const&dec) { base_type::operator=(dec); }
    VarDec(VarDec &dec) : x3::variant<PlainVarDec, InitVarDec>(dec) { }
    VarDec(VarDec const&dec) : x3::variant<PlainVarDec, InitVarDec>(dec) { }
};

struct VarDecStatement: x3::position_tagged {
    DataType type;
    std::vector<VarDec> var_decs;
};

}
}